//! Owner: bijux-analyze
//! Facts export and summary helpers.
use std::fmt::Write;
use std::path::Path;

use anyhow::{Context, Result};
use bijux_core::metrics::ToolInvocationV1;
use bijux_core::primitives::InvariantStatusV1;
use bijux_infra::atomic_write_bytes;
use bijux_runtime::{FactsRowV1, StageReportV1};

use crate::model::{
    stable_sort_records, DashboardFactRow, FactsSummary, JsonBlob, RunSummaryDeltas,
    RunSummaryStageRow, RunSummaryV1,
};

fn stage_report_path(reports: &JsonBlob) -> Option<String> {
    reports
        .as_value()
        .get("stage_report")
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

fn stage_report_for_row(row: &FactsRowV1) -> Option<StageReportV1> {
    let path = stage_report_path(&JsonBlob::from(row.reports.clone()))?;
    let report_raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&report_raw).ok()
}

fn tool_invocation_for_stage(report: &StageReportV1) -> Option<ToolInvocationV1> {
    let raw = std::fs::read_to_string(&report.tool_invocation_path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn params_excerpt(value: &serde_json::Value, limit: usize) -> serde_json::Value {
    let Some(obj) = value.as_object() else {
        return value.clone();
    };
    let mut keys: Vec<_> = obj.keys().cloned().collect();
    keys.sort();
    let mut out = serde_json::Map::new();
    for key in keys.into_iter().take(limit) {
        if let Some(v) = obj.get(&key) {
            out.insert(key, v.clone());
        }
    }
    serde_json::Value::Object(out)
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
            row.input_hash.as_str(),
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
    atomic_write_bytes(path, &serde_json::to_vec_pretty(&payload)?)
        .map_err(anyhow::Error::from)
        .with_context(|| format!("write run summary {}", path.display()))?;
    Ok(())
}

/// Write a deterministic stage summary CSV from facts rows.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_stage_summary_csv(path: &Path, rows: &[FactsRowV1]) -> Result<()> {
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
    let mut output = String::new();
    output.push_str("stage,tool,version,input_reads,output_reads,retention,runtime_s,memory_mb,key_params,verdict,notes\n");
    for row in ordered {
        let retention = match (row.reads_in, row.reads_out) {
            #[allow(clippy::cast_precision_loss)]
            (Some(ri), Some(ro)) if ri > 0 => Some(ro as f64 / ri as f64),
            _ => None,
        };
        let stage_report = stage_report_for_row(&row);
        let verdict = stage_report
            .as_ref()
            .and_then(|report| report.verdict.as_ref())
            .map_or("", |verdict| match verdict.verdict {
                InvariantStatusV1::Pass => "pass",
                InvariantStatusV1::Warn => "warn",
                InvariantStatusV1::Fail => "fail",
            });
        let mut notes = Vec::new();
        if let Some(report) = stage_report.as_ref() {
            if let Some(verdict) = report.verdict.as_ref() {
                notes.extend(verdict.reasons.clone());
            }
            notes.extend(report.warnings.clone());
            notes.extend(report.errors.clone());
        }
        let key_params = stage_report
            .as_ref()
            .and_then(tool_invocation_for_stage)
            .map_or_else(
                || serde_json::json!({}),
                |invocation| {
                    let params = if invocation.effective_params_json_normalized.is_null() {
                        invocation.parameters_json_normalized
                    } else {
                        invocation.effective_params_json_normalized
                    };
                    params_excerpt(&params, 8)
                },
            );
        let key_params = serde_json::to_string(&key_params).unwrap_or_else(|_| "{}".to_string());
        let notes = notes.join(" | ");
        let reads_in = row.reads_in.map(|v| v.to_string()).unwrap_or_default();
        let reads_out = row.reads_out.map(|v| v.to_string()).unwrap_or_default();
        let retention = retention.map(|v| format!("{v:.4}")).unwrap_or_default();
        let _ = writeln!(
            output,
            "{},{},{},{},{},{},{:.2},{:.2},{},{},{}",
            csv_escape(&row.stage_id),
            csv_escape(&row.tool_id),
            csv_escape(&row.tool_version),
            csv_escape(reads_in.as_str()),
            csv_escape(reads_out.as_str()),
            csv_escape(&retention),
            row.runtime_s,
            row.memory_mb,
            csv_escape(&key_params),
            csv_escape(verdict),
            csv_escape(&notes),
        );
    }
    atomic_write_bytes(path, output.as_bytes())
        .map_err(anyhow::Error::from)
        .with_context(|| format!("write stage summary csv {}", path.display()))?;
    Ok(())
}

fn csv_escape(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        let escaped = value.replace('"', "\"\"");
        format!("\"{escaped}\"")
    } else {
        value.to_string()
    }
}

/// Write a deterministic dashboard facts JSONL file.
///
/// # Errors
/// Returns an error if the file cannot be written.
pub fn write_dashboard_facts_jsonl(path: &Path, rows: &[FactsRowV1]) -> Result<()> {
    let mut ordered: Vec<DashboardFactRow> = rows
        .iter()
        .map(|row| DashboardFactRow {
            schema_version: "bijux.dashboard.fact.v1".to_string(),
            run_id: row.run_id.clone(),
            stage_id: row.stage_id.clone(),
            tool_id: row.tool_id.clone(),
            tool_version: row.tool_version.clone(),
            image_digest: row.image_digest.clone(),
            params_hash: row.params_hash.clone(),
            input_hash: row.input_hash.clone(),
            runtime_s: row.runtime_s,
            memory_mb: row.memory_mb,
            exit_code: row.exit_code,
            bank_hashes: JsonBlob::new(row.bank_hashes.clone()),
            metrics: JsonBlob::new(row.metrics.clone()),
            reports: JsonBlob::new(row.reports.clone()),
            artifacts: JsonBlob::new(row.artifacts.clone()),
            trace_id: row.trace_id.clone(),
            span_id: row.span_id.clone(),
        })
        .collect();
    ordered.sort_by(|a, b| a.key().cmp(&b.key()));
    let mut payload = String::new();
    for row in ordered {
        payload.push_str(&serde_json::to_string(&row)?);
        payload.push('\n');
    }
    atomic_write_bytes(path, payload.as_bytes()).map_err(anyhow::Error::from)?;
    Ok(())
}
