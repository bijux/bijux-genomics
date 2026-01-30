use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::run_layout::{RunIndexLine, RunManifest};
use bijux_core::selection::Objective;
use bijux_core::RunMetadataV1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunBenchmarkRecord {
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub read_retention: Option<f64>,
    pub platform: String,
    pub runner: String,
    pub hostname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSummary {
    pub stage: String,
    pub objective: String,
    pub records: Vec<RunBenchmarkRecord>,
    pub ranking: Vec<ToolRanking>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRanking {
    pub tool: String,
    pub score: f64,
    pub runtime_median: Option<f64>,
    pub memory_median: Option<f64>,
    pub retention_median: Option<f64>,
}

/// Benchmark existing runs without re-executing tools.
///
/// # Errors
/// Returns an error if runs or metrics cannot be loaded.
pub fn benchmark_runs(
    runs_dir: &Path,
    stage: &str,
    objective: Objective,
) -> Result<BenchmarkSummary> {
    let base_dir = runs_dir
        .parent()
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let index_path = base_dir.join("bijux-runs").join("index.jsonl");
    if !index_path.exists() {
        return Err(anyhow!(
            "missing bijux-runs/index.jsonl under {}",
            base_dir.display()
        ));
    }
    let mut index_lines = Vec::new();
    for line in std::fs::read_to_string(&index_path)?.lines() {
        let entry: RunIndexLine = serde_json::from_str(line)?;
        index_lines.push(entry.run);
    }

    let mut records = Vec::new();
    for entry in index_lines {
        if !entry.stages.iter().any(|s| s == stage) {
            continue;
        }
        let run_path = runs_dir.join(&entry.run_id);
        let manifest_path = run_path.join("execution_manifest.json");
        let metadata_path = run_path.join("run_metadata.json");
        let manifest: RunManifest =
            serde_json::from_str(&std::fs::read_to_string(&manifest_path)?)?;
        let metadata: RunMetadataV1 =
            serde_json::from_str(&std::fs::read_to_string(&metadata_path)?)?;
        for stage_entry in manifest.stages.iter().filter(|s| s.stage_id == stage) {
            let (runtime_s, memory_mb, retention) = load_metrics(
                &stage_entry.execution_metrics_path,
                &stage_entry.domain_metrics_path,
            )
            .with_context(|| {
                format!(
                    "load metrics for {} {}",
                    stage_entry.stage_id, stage_entry.tool_id
                )
            })?;
            records.push(RunBenchmarkRecord {
                run_id: manifest.run_id.clone(),
                stage_id: stage_entry.stage_id.clone(),
                tool_id: stage_entry.tool_id.clone(),
                runtime_s,
                memory_mb,
                read_retention: retention,
                platform: metadata.platform.clone(),
                runner: metadata.platform.clone(),
                hostname: metadata.hostname.clone(),
            });
        }
    }

    let ranking = rank_records(&records, objective);
    Ok(BenchmarkSummary {
        stage: stage.to_string(),
        objective: objective.as_str().to_string(),
        records,
        ranking,
    })
}

/// Write JSON + CSV benchmark exports.
///
/// # Errors
/// Returns an error if serialization or writing fails.
pub fn write_benchmark_exports(
    runs_dir: &Path,
    summary: &BenchmarkSummary,
) -> Result<(PathBuf, PathBuf)> {
    let json_path = runs_dir.join(format!("benchmark_{}.json", summary.stage));
    let csv_path = runs_dir.join(format!("benchmark_{}.csv", summary.stage));
    std::fs::write(&json_path, serde_json::to_string_pretty(summary)?)?;

    let mut csv = String::new();
    csv.push_str("run_id,stage,tool,runtime_s,memory_mb,read_retention,platform,runner,hostname\n");
    for record in &summary.records {
        use std::fmt::Write;
        let retention = record
            .read_retention
            .map_or_else(|| "n/a".to_string(), |v| format!("{v:.6}"));
        writeln!(
            &mut csv,
            "{},{},{},{:.6},{:.2},{},{},{},{}",
            record.run_id,
            record.stage_id,
            record.tool_id,
            record.runtime_s,
            record.memory_mb,
            retention,
            record.platform,
            record.runner,
            record.hostname
        )?;
    }
    std::fs::write(&csv_path, csv)?;
    Ok((json_path, csv_path))
}

fn rank_records(records: &[RunBenchmarkRecord], objective: Objective) -> Vec<ToolRanking> {
    let mut grouped: BTreeMap<String, Vec<&RunBenchmarkRecord>> = BTreeMap::new();
    for record in records {
        grouped
            .entry(record.tool_id.clone())
            .or_default()
            .push(record);
    }
    let mut rankings = Vec::new();
    for (tool, tool_records) in grouped {
        let runtime_median = median(tool_records.iter().map(|r| r.runtime_s).collect());
        let memory_median = median(tool_records.iter().map(|r| r.memory_mb).collect());
        let retention_median = median(
            tool_records
                .iter()
                .filter_map(|r| r.read_retention)
                .collect(),
        );
        let score = score_for_objective(objective, runtime_median, memory_median, retention_median);
        rankings.push(ToolRanking {
            tool,
            score,
            runtime_median,
            memory_median,
            retention_median,
        });
    }
    rankings.sort_by(|a, b| {
        a.score
            .partial_cmp(&b.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    rankings
}

fn score_for_objective(
    objective: Objective,
    runtime: Option<f64>,
    memory: Option<f64>,
    retention: Option<f64>,
) -> f64 {
    match objective {
        Objective::Speed => runtime.unwrap_or(f64::INFINITY),
        Objective::Memory => memory.unwrap_or(f64::INFINITY),
        Objective::Retention => retention.map_or(f64::INFINITY, |v| -v),
        Objective::Balanced => {
            let runtime = runtime.unwrap_or(f64::INFINITY);
            let memory = memory.unwrap_or(f64::INFINITY);
            let retention = retention.unwrap_or(0.0);
            runtime + memory - (retention * 100.0)
        }
    }
}

fn median(values: Vec<f64>) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values;
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some(f64::midpoint(sorted[mid - 1], sorted[mid]))
    } else {
        Some(sorted[mid])
    }
}

fn load_metrics(execution_path: &Path, domain_path: &Path) -> Result<(f64, f64, Option<f64>)> {
    let execution_data = std::fs::read_to_string(execution_path)?;
    let execution_value: serde_json::Value = serde_json::from_str(&execution_data)?;
    let runtime_s = execution_value
        .get("runtime_s")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("missing runtime_s"))?;
    let memory_mb = execution_value
        .get("memory_mb")
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("missing memory_mb"))?;
    let domain_data = std::fs::read_to_string(domain_path)?;
    let domain_value: serde_json::Value = serde_json::from_str(&domain_data)?;
    let read_retention = domain_value
        .get("metrics")
        .and_then(|v| v.get("delta_metrics"))
        .and_then(|v| v.get("read_retention"))
        .and_then(serde_json::Value::as_f64);
    Ok((runtime_s, memory_mb, read_retention))
}
