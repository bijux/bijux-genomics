//! Owner: bijux-analyze
//! Report sections for run reports.

pub mod schema;

use bijux_core::observability::QcPostReportV1;
use bijux_core::{FactsRowV1, FilterReportV1, RawFailure, ToolInvocationV1};
use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use crate::failure::{classify_raw_failure, BenchmarkFailure};

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
) -> serde_json::Value {
    let mut by_stage: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    for row in rows {
        by_stage.entry(row.stage_id.clone()).or_insert_with(|| {
            let (missing_metrics, missing_reports) = missing_by_stage
                .get(&row.stage_id)
                .cloned()
                .unwrap_or_default();
            serde_json::json!({
                "stage_id": row.stage_id,
                "tool_id": row.tool_id,
                "tool_version": row.tool_version,
                "params_hash": row.params_hash,
                "input_hash": row.input_hash,
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
        "modules": entries,
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
