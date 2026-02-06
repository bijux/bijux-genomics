use std::cmp::Ordering;
use std::fs;

use bijux_core::foundation::InvariantStatusV1;
use bijux_runtime::{FactsRowV1, FilterReportV1, QcPostReportV1};

use super::{artifact_path_for, report_path_for, stage_report_for_row};

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

pub(crate) fn qc_improvement_section(rows: &[FactsRowV1]) -> serde_json::Value {
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

pub(crate) fn qc_artifacts_section(rows: &[FactsRowV1]) -> serde_json::Value {
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
        "fastqc_metrics_v2_path": report.fastqc_metrics_v2_path,
        "top_findings": top_findings_from_invariants(rows, 5),
    })
}

pub(crate) fn filter_interpretation_section(rows: &[FactsRowV1]) -> serde_json::Value {
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

pub(crate) fn adapter_inference_section(rows: &[FactsRowV1]) -> serde_json::Value {
    let mut report_path = None;
    let mut detect_path = None;
    for row in rows {
        if row.stage_id == "fastq.qc_post" {
            report_path = report_path_for(&row.reports, "qc_post_report");
            if report_path.is_some() {
                break;
            }
        }
        if row.stage_id == "fastq.detect_adapters" {
            detect_path = report_path_for(&row.reports, "adapter_candidates");
        }
    }
    let Some(path) = report_path else {
        let detect = detect_path
            .as_deref()
            .and_then(|path| fs::read_to_string(path).ok())
            .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
            .unwrap_or_else(|| serde_json::json!({}));
        return serde_json::json!({
            "suggested_adapters": detect,
            "rationale": "Adapter detection ran before trimming.",
            "safety": "Inference only applies when adapters are not explicitly set.",
        });
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
    let detect = detect_path
        .as_deref()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .unwrap_or_else(|| serde_json::json!({}));
    serde_json::json!({
        "suggested_preset": report.suggested_preset,
        "suggested_adapters": suggestions,
        "detect_adapters": detect,
        "rationale": rationale,
        "safety": "Inference only applies when adapters are not explicitly set.",
    })
}

pub(crate) fn adapter_config_section(rows: &[FactsRowV1]) -> serde_json::Value {
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
