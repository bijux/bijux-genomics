use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use super::{CorrectErrorsReportV1, PairedMode};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyCorrectErrorsReportV1 {
    schema_version: String,
    stage_id: String,
    tool_id: Option<String>,
    tool: Option<String>,
    correction_engine: crate::params::correct::CorrectionEngine,
    quality_encoding: crate::params::correct::QualityEncoding,
    input_r1: String,
    input_r2: Option<String>,
    output_r1: String,
    output_r2: Option<String>,
    kmer_size: Option<u32>,
    musket_kmer_budget: Option<u64>,
    genome_size: Option<u64>,
    max_memory_gb: Option<u32>,
    trusted_kmer_artifact: Option<std::path::PathBuf>,
    conservative_mode: Option<bool>,
    corrected_reads: Option<u64>,
    reads_in: Option<u64>,
    reads_out: Option<u64>,
    bases_in: Option<u64>,
    bases_out: Option<u64>,
    pairs_in: Option<u64>,
    pairs_out: Option<u64>,
    mean_q_before: Option<f64>,
    mean_q_after: Option<f64>,
    kmer_fix_rate: Option<f64>,
    correction_effect: Option<serde_json::Value>,
    runtime_s: Option<f64>,
    memory_mb: Option<f64>,
    exit_code: Option<i32>,
}

fn parse_legacy_correct_errors_report(report_json: &str) -> Result<CorrectErrorsReportV1> {
    let legacy: LegacyCorrectErrorsReportV1 =
        serde_json::from_str(report_json).context("parse legacy correct errors report")?;
    if legacy.schema_version != "bijux.fastq.correct_errors.report.v1" {
        return Err(anyhow!("unsupported correct errors report schema {}", legacy.schema_version));
    }
    Ok(CorrectErrorsReportV1 {
        schema_version: crate::CORRECT_ERRORS_REPORT_SCHEMA_VERSION.to_string(),
        stage: legacy.stage_id.clone(),
        stage_id: legacy.stage_id,
        tool_id: legacy.tool_id.or(legacy.tool).unwrap_or_else(|| "unknown".to_string()),
        paired_mode: PairedMode::from_has_r2(legacy.input_r2.is_some()),
        threads: 1,
        correction_engine: legacy.correction_engine,
        quality_encoding: legacy.quality_encoding,
        kmer_size: legacy.kmer_size,
        musket_kmer_budget: legacy.musket_kmer_budget,
        genome_size: legacy.genome_size,
        max_memory_gb: legacy.max_memory_gb,
        trusted_kmer_artifact: legacy.trusted_kmer_artifact,
        conservative_mode: legacy.conservative_mode.unwrap_or(false),
        input_r1: legacy.input_r1,
        input_r2: legacy.input_r2,
        output_r1: legacy.output_r1,
        output_r2: legacy.output_r2,
        report_json: "correct_report.json".to_string(),
        corrected_reads: legacy.corrected_reads.or(legacy.reads_out),
        changed_reads: None,
        unchanged_reads: None,
        reads_in: legacy.reads_in,
        reads_out: legacy.reads_out,
        bases_in: legacy.bases_in,
        bases_out: legacy.bases_out,
        pairs_in: legacy.pairs_in,
        pairs_out: legacy.pairs_out,
        mean_q_before: legacy.mean_q_before,
        mean_q_after: legacy.mean_q_after,
        kmer_fix_rate: legacy.kmer_fix_rate,
        correction_effect: legacy.correction_effect,
        runtime_s: legacy.runtime_s,
        memory_mb: legacy.memory_mb,
        exit_code: legacy.exit_code,
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: None,
    })
}

/// # Errors
/// Returns an error if the governed correct-errors report JSON cannot be parsed.
pub fn parse_correct_errors_report(report_json: &str) -> Result<CorrectErrorsReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_correct_errors_report(report_json))
        .context("parse correct errors report")
}
