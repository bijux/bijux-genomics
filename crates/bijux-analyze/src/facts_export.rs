//! Owner: bijux-analyze
//! Facts export and summary helpers.
use std::path::Path;

use anyhow::{Context, Result};
use bijux_core::{FactsRowV1, StageReportV1};

use crate::model::JsonBlob;

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
    pub final_outputs: Vec<String>,
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
    pub bank_hashes: JsonBlob,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub reports: JsonBlob,
    pub deltas: RunSummaryDeltas,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunSummaryDeltas {
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
}

pub(crate) fn stable_sort_records<T>(
    rows: &mut [T],
    key: impl Fn(&T) -> (&str, &str, &str, &str, &str),
) {
    rows.sort_by(|a, b| key(a).cmp(&key(b)));
}

fn stage_report_path(reports: &JsonBlob) -> Option<String> {
    reports
        .as_value()
        .get("stage_report")
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn stage_outputs_for_row(row: &FactsRowV1) -> Vec<String> {
    let Some(path) = stage_report_path(&JsonBlob::from(row.reports.clone())) else {
        return Vec::new();
    };
    let Ok(report_raw) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let Ok(report) = serde_json::from_str::<StageReportV1>(&report_raw) else {
        return Vec::new();
    };
    report.outputs
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
            bank_hashes: JsonBlob::from(row.bank_hashes.clone()),
            runtime_s: row.runtime_s,
            memory_mb: row.memory_mb,
            exit_code: row.exit_code,
            reports: JsonBlob::from(row.reports.clone()),
            deltas: RunSummaryDeltas {
                reads_in: row.reads_in,
                reads_out: row.reads_out,
                bases_in: row.bases_in,
                bases_out: row.bases_out,
                pairs_in: row.pairs_in,
                pairs_out: row.pairs_out,
            },
        })
        .collect();
    let mut final_outputs = Vec::new();
    for row in rows {
        if row.stage_id == "fastq.qc_post" {
            final_outputs.extend(stage_outputs_for_row(row));
        }
    }
    final_outputs.sort();
    final_outputs.dedup();
    stable_sort_records(&mut stage_rows, |row| {
        (
            row.run_id.as_str(),
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.params_hash.as_str(),
            "",
        )
    });
    let payload = RunSummaryV1 {
        schema_version: "bijux.run_summary.v1".to_string(),
        facts_path,
        report_path,
        telemetry_path,
        final_outputs,
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

/// Write a deterministic dashboard facts JSONL file.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_dashboard_facts_jsonl(path: &Path, rows: &[FactsRowV1]) -> Result<()> {
    let mut ordered = rows.to_vec();
    stable_sort_records(&mut ordered, |row| {
        (
            row.run_id.as_str(),
            row.stage_id.as_str(),
            row.tool_id.as_str(),
            row.params_hash.as_str(),
            "",
        )
    });
    let mut payload = String::new();
    for row in ordered {
        payload.push_str(&serde_json::to_string(&row)?);
        payload.push('\n');
    }
    std::fs::write(path, payload)?;
    Ok(())
}
