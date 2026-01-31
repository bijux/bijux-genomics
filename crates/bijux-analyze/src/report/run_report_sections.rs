use bijux_core::observability::QcPostReportV1;
use bijux_core::{FactsRowV1, FilterReportV1};
use std::cmp::Ordering;
use std::fs;

use super::run_report::report_path_for;

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
