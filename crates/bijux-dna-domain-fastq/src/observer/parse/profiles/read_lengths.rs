use anyhow::{Context, Result};

use super::super::{u64_to_f64, ProfileReadLengthBinV1, ProfileReadLengthsReportV1};

/// # Errors
/// Returns an error if the governed profile-read-lengths report JSON cannot be parsed.
pub fn parse_profile_read_lengths_report(report_json: &str) -> Result<ProfileReadLengthsReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_profile_read_lengths_report(report_json))
        .context("parse profile read lengths report")
}

fn parse_legacy_profile_read_lengths_report(
    report_json: &str,
) -> Result<ProfileReadLengthsReportV1> {
    let json = serde_json::from_str::<serde_json::Value>(report_json)
        .context("parse legacy profile read lengths json")?;
    let histogram = json
        .get("histogram")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let read_length = entry.get("read_length").and_then(serde_json::Value::as_u64)?;
            let count = entry.get("count").and_then(serde_json::Value::as_u64)?;
            Some(ProfileReadLengthBinV1 { read_length, count })
        })
        .collect::<Vec<_>>();
    let read_count = histogram.iter().map(|bin| bin.count).sum::<u64>();
    let total_length =
        histogram.iter().map(|bin| bin.read_length.saturating_mul(bin.count)).sum::<u64>();
    let mean_read_length =
        if read_count == 0 { 0.0 } else { u64_to_f64(total_length) / u64_to_f64(read_count) };
    let min_read_length = histogram.iter().map(|bin| bin.read_length).min().unwrap_or(0);
    let max_read_length = histogram.iter().map(|bin| bin.read_length).max().unwrap_or(0);
    let median_read_length = histogram_median_read_length(&histogram);
    Ok(ProfileReadLengthsReportV1 {
        schema_version: "bijux.fastq.profile_read_lengths.report.v1_legacy".to_string(),
        stage: "fastq.profile_read_lengths".to_string(),
        stage_id: "fastq.profile_read_lengths".to_string(),
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
        histogram_bins: u32::try_from(histogram.len()).unwrap_or(u32::MAX).max(1),
        input_r1: String::new(),
        input_r2: None,
        length_distribution_tsv: String::new(),
        length_distribution_json: String::new(),
        report_json: String::new(),
        read_count,
        min_read_length,
        mean_read_length,
        median_read_length,
        max_read_length,
        distinct_lengths: histogram.len() as u64,
        histogram,
        runtime_s: json.get("runtime_s").and_then(serde_json::Value::as_f64),
        memory_mb: json.get("memory_mb").and_then(serde_json::Value::as_f64),
        exit_code: json
            .get("exit_code")
            .and_then(serde_json::Value::as_i64)
            .and_then(|value| i32::try_from(value).ok()),
        raw_backend_report: None,
        raw_backend_report_format: None,
    })
}

fn histogram_median_read_length(histogram: &[ProfileReadLengthBinV1]) -> f64 {
    let total_count = histogram.iter().map(|bin| bin.count).sum::<u64>();
    if total_count == 0 {
        return 0.0;
    }

    let midpoint_low = (total_count - 1) / 2;
    let midpoint_high = total_count / 2;
    let mut seen = 0_u64;
    let mut low_value = None;
    let mut high_value = None;

    for bin in histogram {
        let next_seen = seen.saturating_add(bin.count);
        if low_value.is_none() && midpoint_low < next_seen {
            low_value = Some(bin.read_length);
        }
        if high_value.is_none() && midpoint_high < next_seen {
            high_value = Some(bin.read_length);
            break;
        }
        seen = next_seen;
    }

    match (low_value, high_value) {
        (Some(low), Some(high)) => (u64_to_f64(low) + u64_to_f64(high)) / 2.0,
        (Some(low), None) => u64_to_f64(low),
        _ => 0.0,
    }
}
