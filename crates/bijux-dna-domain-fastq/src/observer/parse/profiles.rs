use anyhow::{Context, Result};

use super::{
    u64_to_f64, OverrepresentedSequenceRowV1, ProfileOverrepresentedReportV1,
    ProfileReadLengthBinV1, ProfileReadLengthsReportV1, ProfileReadsHistogramBinV1,
    ProfileReadsMateSummaryV1, ProfileReadsReportV1,
};

/// # Errors
/// Returns an error if the governed profile-reads report JSON cannot be parsed.
pub fn parse_profile_reads_report(report_json: &str) -> Result<ProfileReadsReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_profile_reads_report(report_json))
        .context("parse profile reads report")
}

/// # Errors
/// Returns an error if the governed profile-read-lengths report JSON cannot be parsed.
pub fn parse_profile_read_lengths_report(report_json: &str) -> Result<ProfileReadLengthsReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_profile_read_lengths_report(report_json))
        .context("parse profile read lengths report")
}

/// # Errors
/// Returns an error if the governed overrepresented-sequence report JSON cannot be parsed.
pub fn parse_profile_overrepresented_report(
    report_json: &str,
) -> Result<ProfileOverrepresentedReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_profile_overrepresented_report(report_json))
        .context("parse profile overrepresented report")
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
        reads_total: json
            .get("reads_total")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        bases_total: json
            .get("bases_total")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0),
        mean_q: json
            .get("mean_q")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        gc_percent: json
            .get("gc_percent")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        length_histogram,
        mate_summaries: vec![ProfileReadsMateSummaryV1 {
            label: "reads_r1".to_string(),
            reads: json
                .get("reads_total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
            bases: json
                .get("bases_total")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or(0),
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
            let read_length = entry
                .get("read_length")
                .and_then(serde_json::Value::as_u64)?;
            let count = entry.get("count").and_then(serde_json::Value::as_u64)?;
            Some(ProfileReadLengthBinV1 { read_length, count })
        })
        .collect::<Vec<_>>();
    let read_count = histogram.iter().map(|bin| bin.count).sum::<u64>();
    let total_length = histogram
        .iter()
        .map(|bin| bin.read_length.saturating_mul(bin.count))
        .sum::<u64>();
    let mean_read_length = if read_count == 0 {
        0.0
    } else {
        u64_to_f64(total_length) / u64_to_f64(read_count)
    };
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
        mean_read_length,
        max_read_length: histogram
            .iter()
            .map(|bin| bin.read_length)
            .max()
            .unwrap_or(0),
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

fn parse_legacy_profile_overrepresented_report(
    report_json: &str,
) -> Result<ProfileOverrepresentedReportV1> {
    let json = serde_json::from_str::<serde_json::Value>(report_json)
        .context("parse legacy profile overrepresented json")?;
    let rows = json
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            Some(OverrepresentedSequenceRowV1 {
                sequence: entry
                    .get("sequence")
                    .and_then(serde_json::Value::as_str)?
                    .to_string(),
                count: entry.get("count").and_then(serde_json::Value::as_u64)?,
                fraction: entry
                    .get("fraction")
                    .and_then(serde_json::Value::as_f64)
                    .unwrap_or(0.0),
                flag: entry
                    .get("flag")
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("background")
                    .to_string(),
            })
        })
        .collect::<Vec<_>>();
    Ok(ProfileOverrepresentedReportV1 {
        schema_version: "bijux.fastq.profile_overrepresented.report.v1_legacy".to_string(),
        stage: "fastq.profile_overrepresented_sequences".to_string(),
        stage_id: "fastq.profile_overrepresented_sequences".to_string(),
        tool_id: json
            .get("tool_id")
            .or_else(|| json.get("tool"))
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
        top_k: json
            .get("top_k")
            .and_then(serde_json::Value::as_u64)
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or_else(|| u32::try_from(rows.len()).unwrap_or(u32::MAX).max(1)),
        input_r1: String::new(),
        input_r2: None,
        overrepresented_sequences_tsv: String::new(),
        overrepresented_sequences_json: String::new(),
        report_json: String::new(),
        sequence_count: json
            .get("sequence_count")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(rows.len() as u64),
        flagged_sequences: json
            .get("flagged_sequences")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| {
                rows.iter()
                    .filter(|row| row.flag == "overrepresented")
                    .count() as u64
            }),
        top_fraction: json
            .get("top_fraction")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or_else(|| rows.first().map_or(0.0, |row| row.fraction)),
        rows,
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
