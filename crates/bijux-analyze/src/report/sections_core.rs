// Owner: bijux-analyze
// Report sections for run reports.

use bijux_core::{InvariantStatusV1, RawFailure, ToolInvocationV1};
use bijux_runtime::{FactsRowV1, PipelineVerdictV1, StageReportV1, TelemetryEventV1};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::decision::score::build_rankings;
use crate::failure::{classify_raw_failure, BenchmarkFailure};
use crate::RankInput;
pub(super) use qc::{
    adapter_config_section, adapter_inference_section, filter_interpretation_section,
    qc_artifacts_section, qc_improvement_section,
};

pub(crate) fn report_path_for(reports: &serde_json::Value, key: &str) -> Option<String> {
    reports
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

pub(crate) fn artifact_path_for(artifacts: &serde_json::Value, key: &str) -> Option<String> {
    artifacts
        .get(key)
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

pub(super) fn stage_completeness_table(
    rows: &[FactsRowV1],
    missing_by_stage: &BTreeMap<String, (Vec<String>, Vec<String>)>,
) -> serde_json::Value {
    let mut by_stage = BTreeMap::new();
    for row in rows {
        by_stage.entry(row.stage_id.clone()).or_insert_with(|| {
            let (missing_metrics, missing_reports) = missing_by_stage
                .get(&row.stage_id)
                .cloned()
                .unwrap_or_default();
            serde_json::json!({
                "stage_id": row.stage_id,
                "status": if missing_metrics.is_empty() && missing_reports.is_empty() { "complete" } else { "incomplete" },
                "missing_metrics": missing_metrics,
                "missing_reports": missing_reports,
            })
        });
    }
    let rows: Vec<serde_json::Value> = by_stage.into_values().collect();
    serde_json::json!({ "rows": rows })
}

pub(super) fn decision_trace_section(
    rows: &[FactsRowV1],
    missing_by_stage: &BTreeMap<String, (Vec<String>, Vec<String>)>,
    telemetry_decisions: &BTreeMap<String, Vec<serde_json::Value>>,
) -> serde_json::Value {
    let mut by_stage: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for row in rows {
        by_stage.entry(row.stage_id.clone()).or_insert_with(|| {
            let (missing_metrics, missing_reports) = missing_by_stage
                .get(&row.stage_id)
                .cloned()
                .unwrap_or_default();
            let decisions = telemetry_decisions
                .get(&row.stage_id)
                .cloned()
                .unwrap_or_default();
            serde_json::json!({
                "stage_id": row.stage_id,
                "tool_id": row.tool_id,
                "tool_version": row.tool_version,
                "params_hash": row.params_hash,
                "input_hash": row.input_hash,
                "quality_gate": row.reports.get("quality_gate").cloned().unwrap_or_else(|| serde_json::json!({})),
                "adapter_validation": row.reports.get("adapter_validation").cloned().unwrap_or_else(|| serde_json::json!({})),
                "contaminant_action": row
                    .reports
                    .get("contaminant_action")
                    .cloned()
                    .unwrap_or(serde_json::json!(false)),
                "telemetry_decisions": decisions,
                "missing_metrics": missing_metrics,
                "missing_reports": missing_reports,
            })
        });
    }
    let entries: Vec<serde_json::Value> = by_stage.into_values().collect();
    serde_json::json!({ "entries": entries })
}

pub(super) fn bench_summary_section(base_dir: &Path) -> serde_json::Value {
    let path = base_dir.join("bench").join("summary.json");
    if !path.exists() {
        return serde_json::json!({});
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_else(|| serde_json::json!({}))
}

fn assertion_results_from_row(row: &FactsRowV1) -> Vec<serde_json::Value> {
    row.reports
        .get("assertions")
        .and_then(|value| value.get("results"))
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn stage_confidence_for_row(row: &FactsRowV1) -> (f64, Vec<String>) {
    let mut score = 1.0_f64;
    let mut reasons = Vec::new();
    if row.exit_code != 0 {
        return (0.0, vec!["tool_exit_nonzero".to_string()]);
    }
    let assertions = assertion_results_from_row(row);
    let mut fail_count = 0_u32;
    let mut warn_count = 0_u32;
    for entry in &assertions {
        match entry.get("status").and_then(|value| value.as_str()) {
            Some("fail") => fail_count += 1,
            Some("warn") => warn_count += 1,
            _ => {}
        }
    }
    if fail_count > 0 {
        score -= 0.4 * f64::from(fail_count);
        reasons.push(format!("assertion_failures:{fail_count}"));
    }
    if warn_count > 0 {
        score -= 0.1 * f64::from(warn_count);
        reasons.push(format!("assertion_warnings:{warn_count}"));
    }
    let mut missing = 0_u32;
    if row.reads_in.is_none() {
        missing += 1;
    }
    if row.reads_out.is_none() {
        missing += 1;
    }
    if row.bases_in.is_none() {
        missing += 1;
    }
    if row.bases_out.is_none() {
        missing += 1;
    }
    if missing > 0 {
        score -= 0.05 * f64::from(missing);
        reasons.push(format!("missing_metrics:{missing}"));
    }
    if score < 0.0 {
        score = 0.0;
    }
    (score, reasons)
}

pub(super) fn assertions_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let entries: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            serde_json::json!({
                "stage_id": row.stage_id,
                "tool_id": row.tool_id,
                "results": assertion_results_from_row(row),
            })
        })
        .collect();
    serde_json::json!({ "entries": entries })
}

pub(super) fn stage_confidence_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut entries: Vec<serde_json::Value> = rows
        .iter()
        .map(|row| {
            let (score, reasons) = stage_confidence_for_row(row);
            let bucket = if score >= 0.85 {
                "high"
            } else if score >= 0.6 {
                "medium"
            } else {
                "low"
            };
            serde_json::json!({
                "stage_id": row.stage_id,
                "tool_id": row.tool_id,
                "score": score,
                "bucket": bucket,
                "reasons": reasons,
            })
        })
        .collect();
    entries.sort_by(|a, b| {
        let a_score = a
            .get("score")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let b_score = b
            .get("score")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        b_score
            .partial_cmp(&a_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    serde_json::json!({ "entries": entries })
}

#[allow(clippy::cast_precision_loss)]
fn u64_to_f64(value: u64) -> f64 {
    value as f64
}

pub(super) fn stage_plots_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut entries = Vec::new();
    let mut waterfall = Vec::new();
    for row in rows {
        let read_retention = match (row.reads_in, row.reads_out) {
            (Some(ri), Some(ro)) if ri > 0 => Some(u64_to_f64(ro) / u64_to_f64(ri)),
            _ => None,
        };
        let base_retention = match (row.bases_in, row.bases_out) {
            (Some(bi), Some(bo)) if bi > 0 => Some(u64_to_f64(bo) / u64_to_f64(bi)),
            _ => None,
        };
        let mean_q_delta = row
            .metrics
            .get("mean_q_delta")
            .and_then(serde_json::Value::as_f64)
            .or_else(|| {
                row.metrics
                    .get("delta_metrics")
                    .and_then(|value| value.get("mean_q_delta"))
                    .and_then(serde_json::Value::as_f64)
            });
        let gc_delta = row
            .metrics
            .get("gc_delta")
            .and_then(serde_json::Value::as_f64)
            .or_else(|| {
                row.metrics
                    .get("delta_metrics")
                    .and_then(|value| value.get("gc_delta"))
                    .and_then(serde_json::Value::as_f64)
            });
        entries.push(serde_json::json!({
            "stage_id": row.stage_id,
            "tool_id": row.tool_id,
            "read_retention": read_retention,
            "base_retention": base_retention,
            "mean_q_delta": mean_q_delta,
            "gc_delta": gc_delta,
        }));
        if let Some(value) = read_retention {
            waterfall.push(serde_json::json!({
                "stage_id": row.stage_id,
                "read_retention": value,
            }));
        }
    }
    serde_json::json!({
        "entries": entries,
        "waterfall": waterfall,
    })
}

pub(super) fn reproducibility_section(
    rows: &[FactsRowV1],
    telemetry_paths: &[String],
) -> serde_json::Value {
    let mut tool_versions = Vec::new();
    let mut image_digests = Vec::new();
    let mut params_hashes = Vec::new();
    let mut input_hashes = Vec::new();
    for row in rows {
        tool_versions.push(row.tool_version.clone());
        if let Some(digest) = row.image_digest.clone() {
            image_digests.push(digest);
        }
        params_hashes.push(row.params_hash.clone());
        input_hashes.push(row.input_hash.clone());
    }
    tool_versions.sort();
    tool_versions.dedup();
    image_digests.sort();
    image_digests.dedup();
    params_hashes.sort();
    params_hashes.dedup();
    input_hashes.sort();
    input_hashes.dedup();
    let (started_at, finished_at) = telemetry_bounds(telemetry_paths);
    serde_json::json!({
        "command": "unknown",
        "tool_versions": tool_versions,
        "image_digests": image_digests,
        "params_hashes": params_hashes,
        "input_hashes": input_hashes,
        "started_at": started_at,
        "finished_at": finished_at,
        "determinism_policy": {
            "json_canonicalization": "sorted_keys",
            "html_table_order": "stage_id,tool_id",
            "float_format": "fixed_3",
            "timestamp_format": "rfc3339_utc",
        },
    })
}

fn telemetry_bounds(paths: &[String]) -> (serde_json::Value, serde_json::Value) {
    let mut earliest: Option<String> = None;
    let mut latest: Option<String> = None;
    for path in paths {
        let Ok(raw) = fs::read_to_string(path) else {
            continue;
        };
        for line in raw.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let Ok(event) = serde_json::from_str::<TelemetryEventV1>(line) else {
                continue;
            };
            let ts = event.timestamp;
            if earliest.as_ref().map_or(true, |curr| ts < *curr) {
                earliest = Some(ts.clone());
            }
            if latest.as_ref().map_or(true, |curr| ts > *curr) {
                latest = Some(ts.clone());
            }
        }
    }
    (
        earliest.map_or(serde_json::Value::Null, serde_json::Value::String),
        latest.map_or(serde_json::Value::Null, serde_json::Value::String),
    )
}

pub(super) fn scientific_provenance_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut entries = Vec::new();
    for row in rows {
        let mut effective_params = serde_json::json!({});
        let mut raw_params = serde_json::json!({});
        let mut resolved_tool_version = None;
        if let Some(stage_report_path) = report_path_for(&row.reports, "stage_report") {
            if let Some(stage_report_value) = read_json_value(Path::new(&stage_report_path)) {
                if let Ok(report) = serde_json::from_value::<StageReportV1>(stage_report_value) {
                    if let Some(invocation_value) =
                        read_json_value(Path::new(&report.tool_invocation_path))
                    {
                        if let Ok(invocation) =
                            serde_json::from_value::<ToolInvocationV1>(invocation_value)
                        {
                            resolved_tool_version = invocation.resolved_tool_version;
                        }
                    }
                    if let Some(config_value) =
                        read_json_value(Path::new(&report.effective_config_path))
                    {
                        if let Some(value) = config_value.get("effective_params_json") {
                            effective_params = value.clone();
                        }
                        if let Some(value) = config_value.get("parameters_json") {
                            raw_params = value.clone();
                        }
                    }
                }
            }
        }
        entries.push(serde_json::json!({
            "stage_id": row.stage_id,
            "tool_id": row.tool_id,
            "tool_version": row.tool_version,
            "resolved_tool_version": resolved_tool_version,
            "image_digest": row.image_digest,
            "params_hash": row.params_hash,
            "input_hash": row.input_hash,
            "output_hashes": row.output_hashes,
            "effective_params": effective_params,
            "raw_params": raw_params,
        }));
    }
    serde_json::json!({ "entries": entries })
}
