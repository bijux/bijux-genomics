use anyhow::{Context, Result};

use super::super::{OverrepresentedSequenceRowV1, ProfileOverrepresentedReportV1};

/// # Errors
/// Returns an error if the governed overrepresented-sequence report JSON cannot be parsed.
pub fn parse_profile_overrepresented_report(
    report_json: &str,
) -> Result<ProfileOverrepresentedReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_profile_overrepresented_report(report_json))
        .context("parse profile overrepresented report")
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
