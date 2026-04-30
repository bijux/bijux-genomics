use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use super::{parse_report_u64_field, ExtractUmisReportV1, FilterLowComplexityReportV1};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyFilterLowComplexityReportV1 {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    input_fastq: String,
    output_fastq: String,
    #[serde(default)]
    output_fastq_r2: Option<String>,
    reads_in: u64,
    reads_out: u64,
    reads_removed_low_complexity: u64,
    #[serde(default)]
    runtime_s: Option<f64>,
    #[serde(default)]
    memory_mb: Option<f64>,
    #[serde(default)]
    exit_code: Option<i32>,
}

fn parse_legacy_filter_low_complexity_report(
    report_json: &str,
) -> Result<FilterLowComplexityReportV1> {
    let legacy: LegacyFilterLowComplexityReportV1 =
        serde_json::from_str(report_json).context("parse legacy low complexity report")?;
    if legacy.schema_version != "bijux.fastq.filter_low_complexity.report.v1" {
        return Err(anyhow!("unsupported low-complexity report schema {}", legacy.schema_version));
    }
    let paired_mode = if legacy.output_fastq_r2.is_some() {
        crate::PairedMode::PairedEnd
    } else {
        crate::PairedMode::SingleEnd
    };
    Ok(FilterLowComplexityReportV1 {
        schema_version: crate::FILTER_LOW_COMPLEXITY_REPORT_SCHEMA_VERSION.to_string(),
        stage: legacy.stage_id.clone(),
        stage_id: legacy.stage_id,
        tool_id: legacy.tool_id,
        paired_mode,
        threads: 1,
        input_r1: legacy.input_fastq,
        input_r2: None,
        output_r1: legacy.output_fastq,
        output_r2: legacy.output_fastq_r2,
        report_json: "low_complexity_report.json".to_string(),
        entropy_threshold: None,
        polyx_threshold: None,
        reads_in: legacy.reads_in,
        reads_out: legacy.reads_out,
        reads_removed_low_complexity: legacy.reads_removed_low_complexity,
        bases_in: 0,
        bases_out: 0,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 0.0,
        mean_q_after: 0.0,
        runtime_s: legacy.runtime_s,
        memory_mb: legacy.memory_mb,
        exit_code: legacy.exit_code,
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: None,
    })
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyExtractUmisReportV1 {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    input_r1: String,
    input_r2: String,
    output_r1: String,
    output_r2: String,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    #[serde(default)]
    pairs_in: Option<u64>,
    #[serde(default)]
    pairs_out: Option<u64>,
    reads_with_umi: u64,
    runtime_s: f64,
    memory_mb: f64,
    exit_code: i32,
}

fn parse_legacy_extract_umis_report(report_json: &str) -> Result<ExtractUmisReportV1> {
    let legacy: LegacyExtractUmisReportV1 =
        serde_json::from_str(report_json).context("parse legacy extract umis report")?;
    if legacy.schema_version != "bijux.fastq.extract_umis.report.v1" {
        return Err(anyhow!("unsupported extract-umis report schema {}", legacy.schema_version));
    }
    Ok(ExtractUmisReportV1 {
        schema_version: crate::EXTRACT_UMIS_REPORT_SCHEMA_VERSION.to_string(),
        stage: legacy.stage_id.clone(),
        stage_id: legacy.stage_id,
        tool_id: legacy.tool_id,
        paired_mode: crate::PairedMode::PairedEnd,
        threads: 1,
        umi_pattern: String::new(),
        extraction_location: crate::params::umi::UmiExtractionLocation::Read1Prefix,
        read_name_transform: crate::params::umi::UmiReadNameTransform::AppendToHeader,
        failed_extraction_policy: crate::params::umi::UmiFailedExtractionPolicy::RefuseStage,
        downstream_propagation: crate::params::umi::UmiDownstreamPropagation::HeaderAndReport,
        input_r1: legacy.input_r1,
        input_r2: Some(legacy.input_r2),
        output_r1: legacy.output_r1,
        output_r2: Some(legacy.output_r2),
        report_json: "umi_report.json".to_string(),
        reads_in: legacy.reads_in,
        reads_out: legacy.reads_out,
        bases_in: legacy.bases_in,
        bases_out: legacy.bases_out,
        pairs_in: legacy.pairs_in,
        pairs_out: legacy.pairs_out,
        reads_with_umi: legacy.reads_with_umi,
        failed_extractions: None,
        mean_q_before: 0.0,
        mean_q_after: 0.0,
        runtime_s: Some(legacy.runtime_s),
        memory_mb: Some(legacy.memory_mb),
        exit_code: Some(legacy.exit_code),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: None,
    })
}

/// # Errors
/// Returns an error if the governed extract-umis report JSON cannot be parsed.
pub fn parse_extract_umis_report(report_json: &str) -> Result<ExtractUmisReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_extract_umis_report(report_json))
        .context("parse extract umis report")
}

/// # Errors
/// Returns an error if the governed filter-low-complexity report JSON cannot be parsed.
pub fn parse_filter_low_complexity_report(
    report_json: &str,
) -> Result<FilterLowComplexityReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_filter_low_complexity_report(report_json))
        .context("parse filter low complexity report")
}

/// # Errors
/// Returns an error if report JSON cannot be parsed.
pub fn parse_low_complexity_report(report_json: &str) -> Result<u64> {
    if let Ok(report) = parse_filter_low_complexity_report(report_json) {
        return Ok(report.reads_removed_low_complexity);
    }
    parse_report_u64_field(report_json, "reads_removed_low_complexity")
        .or_else(|| parse_bbduk_reads_removed(report_json).ok())
        .ok_or_else(|| anyhow!("low-complexity report missing reads_removed_low_complexity"))
}

/// # Errors
/// Returns an error if `BBDuk` stats cannot be reduced to a reads-removed count.
pub fn parse_bbduk_reads_removed(stats_txt: &str) -> Result<u64> {
    for line in stats_txt.lines() {
        let line = line.trim();
        if line.starts_with("Reads Removed") || line.starts_with("Reads removed") {
            let digits: String = line.chars().filter(char::is_ascii_digit).collect();
            if !digits.is_empty() {
                return digits.parse::<u64>().context("parse bbduk reads removed");
            }
        }
        if line.starts_with("#Matched") || line.starts_with("Matched") {
            if let Some(field) = line.split_whitespace().nth(1) {
                let digits: String = field.chars().filter(char::is_ascii_digit).collect();
                if !digits.is_empty() {
                    return digits.parse::<u64>().context("parse bbduk matched reads");
                }
            }
        }
    }
    Err(anyhow!("bbduk stats missing reads removed line"))
}
