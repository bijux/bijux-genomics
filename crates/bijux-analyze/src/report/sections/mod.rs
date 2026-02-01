//! Owner: bijux-analyze
//! Report sections for run reports.

pub mod schema;

use bijux_core::observability::QcPostReportV1;
use bijux_core::{
    FactsRowV1, FilterReportV1, InvariantStatusV1, PipelineVerdictV1, RawFailure, StageReportV1,
    TelemetryEventV1, ToolInvocationV1,
};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::decision::score::build_rankings;
use crate::failure::{classify_raw_failure, BenchmarkFailure};
use crate::RankInput;

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
            if earliest.as_ref().is_none_or(|curr| ts < *curr) {
                earliest = Some(ts.clone());
            }
            if latest.as_ref().is_none_or(|curr| ts > *curr) {
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

fn read_json_value(path: &Path) -> Option<serde_json::Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

pub(super) fn pipeline_verdict_from_rows(rows: &[FactsRowV1]) -> PipelineVerdictV1 {
    let mut verdict = InvariantStatusV1::Pass;
    let mut reasons = Vec::new();
    for row in rows {
        let Some(stage_report_path) = report_path_for(&row.reports, "stage_report") else {
            continue;
        };
        let Some(stage_report_value) = read_json_value(Path::new(&stage_report_path)) else {
            continue;
        };
        let Ok(report) = serde_json::from_value::<StageReportV1>(stage_report_value) else {
            continue;
        };
        let Some(stage_verdict) = report.verdict else {
            continue;
        };
        verdict = std::cmp::max(verdict, stage_verdict.verdict.clone());
        if stage_verdict.verdict != InvariantStatusV1::Pass {
            reasons.push(format!(
                "{}:{:?}",
                stage_verdict.stage_id, stage_verdict.verdict
            ));
        }
    }
    PipelineVerdictV1 { verdict, reasons }
}

pub(super) fn pipeline_verdict_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let verdict = pipeline_verdict_from_rows(rows);
    serde_json::to_value(verdict).unwrap_or_else(|_| serde_json::json!({}))
}

pub(super) fn comparison_view_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut by_stage: BTreeMap<String, Vec<&FactsRowV1>> = BTreeMap::new();
    for row in rows {
        by_stage.entry(row.stage_id.clone()).or_default().push(row);
    }
    let mut stages = Vec::new();
    for (stage_id, entries) in by_stage {
        if entries.len() < 2 {
            continue;
        }
        let mut tools = Vec::new();
        let mut rank_inputs = Vec::new();
        for row in entries {
            let read_retention = match (row.reads_in, row.reads_out) {
                #[allow(clippy::cast_precision_loss)]
                (Some(ri), Some(ro)) if ri > 0 => Some(ro as f64 / ri as f64),
                _ => None,
            };
            let base_retention = match (row.bases_in, row.bases_out) {
                #[allow(clippy::cast_precision_loss)]
                (Some(bi), Some(bo)) if bi > 0 => Some(bo as f64 / bi as f64),
                _ => None,
            };
            let error_reduction_proxy = row
                .metrics
                .get("mean_q_delta")
                .and_then(serde_json::Value::as_f64);
            rank_inputs.push(RankInput {
                tool: row.tool_id.clone(),
                runtime_s: row.runtime_s,
                memory_mb: row.memory_mb,
                read_retention,
                base_retention,
                error_reduction_proxy,
            });

            let stage_report = stage_report_for_row(row);
            let verdict = stage_report
                .as_ref()
                .and_then(|report| report.verdict.as_ref())
                .map(|verdict| verdict.verdict.clone());
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

            tools.push(serde_json::json!({
                "tool_id": row.tool_id,
                "tool_version": row.tool_version,
                "params_hash": row.params_hash,
                "runtime_s": row.runtime_s,
                "memory_mb": row.memory_mb,
                "read_retention": read_retention,
                "base_retention": base_retention,
                "verdict": verdict,
                "key_params": key_params,
                "notes": notes,
            }));
        }

        let recommended = build_rankings(&rank_inputs)
            .ok()
            .and_then(|rankings| rankings.get("BalancedPareto").cloned())
            .and_then(|entries| entries.first().map(|entry| entry.tool.clone()));
        stages.push(serde_json::json!({
            "stage_id": stage_id,
            "tools": tools,
            "recommended_default": recommended,
        }));
    }
    serde_json::json!({
        "stages": stages,
    })
}

pub(super) fn failure_hints_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut failures: Vec<BenchmarkFailure> = Vec::new();
    for row in rows {
        if let Some(failures_value) = row.reports.get("failures") {
            if let Some(array) = failures_value.as_array() {
                for entry in array {
                    if let Ok(raw) = serde_json::from_value::<RawFailure>(entry.clone()) {
                        failures.push(classify_raw_failure(&raw));
                    }
                }
            }
        }
    }
    serde_json::json!({
        "failures": failures,
        "count": failures.len(),
    })
}

pub(super) fn read_tool_invocation(path: &Path) -> Option<ToolInvocationV1> {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
}

pub(super) fn params_excerpt(value: &serde_json::Value, limit: usize) -> serde_json::Value {
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

fn stage_report_for_row(row: &FactsRowV1) -> Option<StageReportV1> {
    let path = report_path_for(&row.reports, "stage_report")?;
    let report_raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&report_raw).ok()
}

fn tool_invocation_for_stage(report: &StageReportV1) -> Option<ToolInvocationV1> {
    let invocation_raw = fs::read_to_string(&report.tool_invocation_path).ok()?;
    serde_json::from_str(&invocation_raw).ok()
}

fn top_findings_from_invariants(rows: &[FactsRowV1], limit: usize) -> Vec<serde_json::Value> {
    let mut findings: Vec<(InvariantStatusV1, serde_json::Value)> = Vec::new();
    for row in rows {
        let Some(stage_report) = stage_report_for_row(row) else {
            continue;
        };
        for invariant in stage_report.invariants {
            if invariant.status == InvariantStatusV1::Pass {
                continue;
            }
            findings.push((
                invariant.status.clone(),
                serde_json::json!({
                    "stage_id": row.stage_id,
                    "id": invariant.id,
                    "status": invariant.status,
                    "message": invariant.message,
                    "remediation": invariant.remediation,
                }),
            ));
        }
    }
    findings.sort_by(|a, b| b.0.cmp(&a.0));
    findings
        .into_iter()
        .take(limit)
        .map(|(_, value)| value)
        .collect()
}

pub(super) fn qc_improvement_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut report_path = None;
    for row in rows {
        if row.stage_id == "fastq.qc_post" {
            report_path = report_path_for(&row.reports, "qc_post_report");
            if report_path.is_some() {
                break;
            }
        }
    }
    let Some(path) = report_path else {
        return serde_json::json!({});
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return serde_json::json!({});
    };
    let Ok(report) = serde_json::from_str::<QcPostReportV1>(&raw) else {
        return serde_json::json!({});
    };
    let module_names = [
        "Per base sequence quality",
        "Per sequence quality scores",
        "Per sequence GC content",
        "Adapter Content",
        "Sequence Duplication Levels",
        "Overrepresented sequences",
    ];
    let mut entries = serde_json::Map::new();
    for module in module_names {
        let raw_status = report
            .fastqc_raw_modules
            .get(module)
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let trimmed_status = report
            .fastqc_trimmed_modules
            .get(module)
            .and_then(|value| value.as_str())
            .map(str::to_string);
        let delta = match (raw_status.as_deref(), trimmed_status.as_deref()) {
            (Some(raw), Some(trimmed)) => {
                let score = |status: &str| match status {
                    "PASS" => 2,
                    "WARN" => 1,
                    _ => 0,
                };
                let before = score(raw);
                let after = score(trimmed);
                match after.cmp(&before) {
                    Ordering::Greater => "improved",
                    Ordering::Less => "regressed",
                    Ordering::Equal => "unchanged",
                }
            }
            _ => "unknown",
        };
        entries.insert(
            module.to_string(),
            serde_json::json!({
                "raw_status": raw_status,
                "trimmed_status": trimmed_status,
                "delta": delta,
            }),
        );
    }
    serde_json::json!({
        "raw_fastqc_dir": report.raw_fastqc_dir,
        "trimmed_fastqc_dir": report.trimmed_fastqc_dir,
        "multiqc_report": report.multiqc_report,
        "multiqc_data": report.multiqc_data,
        "top_findings": top_findings_from_invariants(rows, 5),
        "modules": entries,
    })
}

pub(super) fn qc_artifacts_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut report_path = None;
    for row in rows {
        if row.stage_id == "fastq.qc_post" {
            report_path = report_path_for(&row.reports, "qc_post_report");
            if report_path.is_some() {
                break;
            }
        }
    }
    let Some(path) = report_path else {
        return serde_json::json!({});
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return serde_json::json!({});
    };
    let Ok(report) = serde_json::from_str::<QcPostReportV1>(&raw) else {
        return serde_json::json!({});
    };
    serde_json::json!({
        "raw_fastqc_dir": report.raw_fastqc_dir,
        "trimmed_fastqc_dir": report.trimmed_fastqc_dir,
        "multiqc_report": report.multiqc_report,
        "multiqc_data": report.multiqc_data,
        "fastqc_raw_modules": report.fastqc_raw_modules,
        "fastqc_trimmed_modules": report.fastqc_trimmed_modules,
        "top_findings": top_findings_from_invariants(rows, 5),
    })
}

pub(super) fn filter_interpretation_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut report_path = None;
    for row in rows {
        if row.stage_id == "fastq.filter" {
            report_path = report_path_for(&row.reports, "filter_report");
            if report_path.is_some() {
                break;
            }
        }
    }
    let Some(path) = report_path else {
        return serde_json::json!({});
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return serde_json::json!({});
    };
    let Ok(report) = serde_json::from_str::<FilterReportV1>(&raw) else {
        return serde_json::json!({});
    };
    serde_json::json!({
        "why_this_matters": "Filtering removes reads that are likely to be low-quality, low-complexity, or contaminant-derived, improving downstream accuracy.",
        "recommended_ranges": {
            "ancient_dna": {
                "max_n": "0-2",
                "low_complexity_threshold": "0.4-0.6",
                "kmer_ref": "enable if contaminant references are available",
            },
            "modern_ngs": {
                "max_n": "0",
                "low_complexity_threshold": "0.6-0.8",
                "kmer_ref": "enable for known contaminant panels (e.g. PhiX/UniVec)",
            }
        },
        "conditions": report.conditions,
        "removed_breakdown": {
            "total": report.reads_removed_total,
            "by_n": report.reads_removed_by_n,
            "by_entropy": report.reads_removed_by_entropy,
            "by_low_complexity": report.reads_removed_low_complexity,
            "by_kmer": report.reads_removed_by_kmer,
            "by_contaminant_kmer": report.reads_removed_contaminant_kmer,
            "by_length": report.reads_removed_by_length,
        },
        "entropy_distribution": report.entropy_distribution,
        "redundant_filters": report.redundant_filters,
    })
}

pub(super) fn adapter_inference_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut report_path = None;
    for row in rows {
        if row.stage_id == "fastq.qc_post" {
            report_path = report_path_for(&row.reports, "qc_post_report");
            if report_path.is_some() {
                break;
            }
        }
    }
    let Some(path) = report_path else {
        return serde_json::json!({});
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return serde_json::json!({});
    };
    let Ok(report) = serde_json::from_str::<QcPostReportV1>(&raw) else {
        return serde_json::json!({});
    };
    let suggestions = report
        .suggested_adapters_path
        .as_deref()
        .and_then(|path| {
            let raw = fs::read_to_string(path).ok()?;
            serde_json::from_str::<serde_json::Value>(&raw).ok()
        })
        .unwrap_or_else(|| serde_json::json!({}));
    let rationale = match report.suggested_preset.as_deref() {
        Some("illumina_twocolor") => {
            "PolyG/overrepresented sequences consistent with two-color chemistry."
        }
        Some("ssdna") => "Overrepresented adapter motifs match ssDNA library prep.",
        Some(_) => "Adapter motifs detected in overrepresented sequences.",
        None => "No strong adapter signal detected.",
    };
    serde_json::json!({
        "suggested_preset": report.suggested_preset,
        "suggested_adapters": suggestions,
        "rationale": rationale,
        "safety": "Inference never changes trimming unless --accept-suggested-adapters is set.",
    })
}

pub(super) fn adapter_config_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut effective_path = None;
    for row in rows {
        effective_path = artifact_path_for(&row.artifacts, "effective_adapters");
        if effective_path.is_some() {
            break;
        }
    }
    let Some(path) = effective_path else {
        return serde_json::json!({});
    };
    let Ok(raw) = fs::read_to_string(&path) else {
        return serde_json::json!({});
    };
    let Ok(effective) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return serde_json::json!({});
    };
    serde_json::json!({
        "effective_adapters_path": path,
        "effective_adapters": effective,
    })
}
