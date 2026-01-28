use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::objective::Objective;
use super::run_layout::{RunEnvironment, RunIndex, RunManifest};

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
    let index_path = runs_dir.join("index.json");
    if !index_path.exists() {
        return Err(anyhow!(
            "missing runs/index.json under {}",
            runs_dir.display()
        ));
    }
    let index: RunIndex = serde_json::from_str(&std::fs::read_to_string(&index_path)?)?;

    let mut records = Vec::new();
    for entry in index.runs {
        if !entry.stages.iter().any(|s| s == stage) {
            continue;
        }
        let run_path = runs_dir.join(&entry.run_id);
        let manifest_path = run_path.join("run_manifest.json");
        let env_path = run_path.join("environment.json");
        let manifest: RunManifest =
            serde_json::from_str(&std::fs::read_to_string(&manifest_path)?)?;
        let env: RunEnvironment = serde_json::from_str(&std::fs::read_to_string(&env_path)?)?;
        for stage_entry in manifest.stages.iter().filter(|s| s.stage_id == stage) {
            let (runtime_s, memory_mb, retention) = load_metrics(&stage_entry.metrics_path)
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
                platform: env.platform.clone(),
                runner: env.runner.clone(),
                hostname: env.hostname.clone(),
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

fn load_metrics(path: &Path) -> Result<(f64, f64, Option<f64>)> {
    let data = std::fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&data)?;
    let runtime_s = value
        .get("execution")
        .and_then(|v| v.get("runtime_s"))
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("missing runtime_s"))?;
    let memory_mb = value
        .get("execution")
        .and_then(|v| v.get("memory_mb"))
        .and_then(serde_json::Value::as_f64)
        .ok_or_else(|| anyhow!("missing memory_mb"))?;
    let read_retention = value
        .get("metrics")
        .and_then(|v| v.get("delta_metrics"))
        .and_then(|v| v.get("read_retention"))
        .and_then(serde_json::Value::as_f64);
    Ok((runtime_s, memory_mb, read_retention))
}
