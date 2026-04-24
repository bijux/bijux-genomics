use anyhow::{Context, Result};

use super::super::{ProfileReadsHistogramBinV1, ProfileReadsMateSummaryV1, ProfileReadsReportV1};

/// # Errors
/// Returns an error if the governed profile-reads report JSON cannot be parsed.
pub fn parse_profile_reads_report(report_json: &str) -> Result<ProfileReadsReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_profile_reads_report(report_json))
        .context("parse profile reads report")
}

fn parse_legacy_profile_reads_report(report_json: &str) -> Result<ProfileReadsReportV1> {
    let json = serde_json::from_str::<serde_json::Value>(report_json)
        .context("parse legacy profile reads json")?;
    let length_histogram = json
        .get("length_histogram")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let length = entry.get("length").and_then(serde_json::Value::as_u64)?;
            let count = entry.get("count").and_then(serde_json::Value::as_u64)?;
            Some(ProfileReadsHistogramBinV1 { length, count })
        })
        .collect::<Vec<_>>();
    Ok(ProfileReadsReportV1 {
        schema_version: "bijux.fastq.profile_reads.report.v1_legacy".to_string(),
        stage: "fastq.profile_reads".to_string(),
        stage_id: "fastq.profile_reads".to_string(),
        tool_id: json
            .get("tool_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        paired_mode: match json.get("paired_mode").and_then(serde_json::Value::as_str) {
            Some("paired_end") => crate::PairedMode::PairedEnd,
            _ => crate::PairedMode::SingleEnd,
        },
        threads: json
            .get("threads")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        input_r1: String::new(),
        input_r2: None,
        qc_json: String::new(),
        qc_tsv: String::new(),
        qc_plots_dir: None,
        length_histogram_source: "legacy_qc_json".to_string(),
        reads_total: json.get("reads_total").and_then(serde_json::Value::as_u64).unwrap_or(0),
        bases_total: json.get("bases_total").and_then(serde_json::Value::as_u64).unwrap_or(0),
        mean_q: json.get("mean_q").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
        gc_percent: json.get("gc_percent").and_then(serde_json::Value::as_f64).unwrap_or(0.0),
        length_histogram,
        mate_summaries: vec![ProfileReadsMateSummaryV1 {
            label: "reads_r1".to_string(),
            reads: json.get("reads_total").and_then(serde_json::Value::as_u64).unwrap_or(0),
            bases: json.get("bases_total").and_then(serde_json::Value::as_u64).unwrap_or(0),
            mean_q: json.get("mean_q").and_then(serde_json::Value::as_f64),
            gc_percent: json.get("gc_percent").and_then(serde_json::Value::as_f64),
        }],
        runtime_s: json.get("runtime_s").and_then(serde_json::Value::as_f64),
        memory_mb: json.get("memory_mb").and_then(serde_json::Value::as_f64),
        exit_code: json
            .get("exit_code")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok()),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: None,
    })
}
