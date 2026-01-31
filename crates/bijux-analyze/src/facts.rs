use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use anyhow::{Context, Result};
use bijux_core::FactsRowV1;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FactsSummary {
    pub runs: usize,
    pub stages: usize,
    pub total_runtime_s: f64,
    pub avg_runtime_s: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunSummaryV1 {
    pub schema_version: String,
    pub facts_path: Option<String>,
    pub report_path: Option<String>,
    pub telemetry_path: Option<String>,
    pub runs: usize,
    pub stages: usize,
    pub total_runtime_s: f64,
    pub avg_runtime_s: f64,
    pub stage_rows: Vec<RunSummaryStageRow>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunSummaryStageRow {
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub params_hash: String,
    pub input_hash: String,
    pub bank_hashes: serde_json::Value,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub reports: serde_json::Value,
    pub deltas: serde_json::Value,
}

/// Load facts rows from a jsonl file.
///
/// # Errors
/// Returns an error if the file cannot be read or parsed.
pub fn load_facts_jsonl(path: &Path) -> Result<Vec<FactsRowV1>> {
    let file = File::open(path).with_context(|| format!("open facts {}", path.display()))?;
    let reader = BufReader::new(file);
    let mut rows = Vec::new();
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let row: FactsRowV1 = serde_json::from_str(&line)?;
        rows.push(row);
    }
    Ok(rows)
}

#[must_use]
pub fn summarize_facts(rows: &[FactsRowV1]) -> FactsSummary {
    let stages = rows.len();
    let mut run_ids = std::collections::BTreeSet::new();
    let mut total_runtime_s = 0.0;
    for row in rows {
        run_ids.insert(row.run_id.clone());
        total_runtime_s += row.runtime_s;
    }
    let runs = run_ids.len();
    let avg_runtime_s = if stages == 0 {
        0.0
    } else {
        let denom = f64::from(u32::try_from(stages).unwrap_or(u32::MAX));
        total_runtime_s / denom
    };
    FactsSummary {
        runs,
        stages,
        total_runtime_s,
        avg_runtime_s,
    }
}

/// Write a deterministic run summary JSON from facts rows.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_run_summary_json(path: &Path, rows: &[FactsRowV1]) -> Result<()> {
    let summary = summarize_facts(rows);
    let facts_path = Some("facts.jsonl".to_string());
    let report_path = Some("report.json".to_string());
    let telemetry_path = Some("telemetry/events.jsonl".to_string());
    let mut stage_rows: Vec<RunSummaryStageRow> = rows
        .iter()
        .map(|row| RunSummaryStageRow {
            run_id: row.run_id.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            tool_version: row.tool_version.clone(),
            image_digest: row.image_digest.clone(),
            params_hash: row.params_hash.clone(),
            input_hash: row.input_hash.clone(),
            bank_hashes: row.bank_hashes.clone(),
            runtime_s: row.runtime_s,
            memory_mb: row.memory_mb,
            exit_code: row.exit_code,
            reports: row.reports.clone(),
            deltas: serde_json::json!({
                "reads_in": row.reads_in,
                "reads_out": row.reads_out,
                "bases_in": row.bases_in,
                "bases_out": row.bases_out,
                "pairs_in": row.pairs_in,
                "pairs_out": row.pairs_out,
            }),
        })
        .collect();
    stage_rows.sort_by(|a, b| {
        (a.run_id.clone(), a.stage_id.clone(), a.tool_id.clone()).cmp(&(
            b.run_id.clone(),
            b.stage_id.clone(),
            b.tool_id.clone(),
        ))
    });
    let payload = RunSummaryV1 {
        schema_version: "bijux.run_summary.v1".to_string(),
        facts_path,
        report_path,
        telemetry_path,
        runs: summary.runs,
        stages: summary.stages,
        total_runtime_s: summary.total_runtime_s,
        avg_runtime_s: summary.avg_runtime_s,
        stage_rows,
    };
    std::fs::write(path, serde_json::to_vec_pretty(&payload)?)
        .with_context(|| format!("write run summary {}", path.display()))?;
    Ok(())
}
