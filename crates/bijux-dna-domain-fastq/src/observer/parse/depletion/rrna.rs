use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use super::super::{DepleteRrnaReportV1, PairedMode};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyDepleteRrnaReportV1 {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    rrna_fraction_removed: f64,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    #[serde(default)]
    runtime_s: Option<f64>,
    #[serde(default)]
    memory_mb: Option<f64>,
}

fn parse_legacy_deplete_rrna_report(report_json: &str) -> Result<DepleteRrnaReportV1> {
    let legacy: LegacyDepleteRrnaReportV1 =
        serde_json::from_str(report_json).context("parse legacy deplete rrna report")?;
    if legacy.schema_version != "bijux.fastq.deplete_rrna.report.v1" {
        return Err(anyhow!("unsupported deplete rrna report schema {}", legacy.schema_version));
    }
    Ok(DepleteRrnaReportV1 {
        schema_version: crate::DEPLETE_RRNA_REPORT_SCHEMA_VERSION.to_string(),
        stage: legacy.stage_id.clone(),
        stage_id: legacy.stage_id,
        tool_id: legacy.tool_id,
        paired_mode: PairedMode::SingleEnd,
        threads: 1,
        rrna_db: None,
        database_artifact_id: "legacy_rrna_db".to_string(),
        database_build_id: None,
        database_digest: None,
        screening_engine: crate::params::screen::RrnaScreeningEngine::Sortmerna,
        report_format: crate::params::screen::RrnaReportFormat::SummaryTsvAndJson,
        emit_removed_reads: false,
        min_identity: None,
        retained_read_role: "rrna_filtered_reads".to_string(),
        rejected_read_role: "removed_rrna_reads".to_string(),
        input_r1: String::new(),
        input_r2: None,
        output_r1: String::new(),
        output_r2: None,
        rrna_report_tsv: "rrna_report.tsv".to_string(),
        rrna_report_json: "rrna_report.json".to_string(),
        reads_in: legacy.reads_in,
        reads_out: legacy.reads_out,
        reads_removed: legacy.reads_in.saturating_sub(legacy.reads_out),
        bases_in: legacy.bases_in,
        bases_out: legacy.bases_out,
        bases_removed: legacy.bases_in.saturating_sub(legacy.bases_out),
        pairs_in: None,
        pairs_out: None,
        rrna_fraction_removed: legacy.rrna_fraction_removed,
        runtime_s: legacy.runtime_s,
        memory_mb: legacy.memory_mb,
        exit_code: None,
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: None,
    })
}

/// # Errors
/// Returns an error if the governed rrna-depletion report JSON cannot be parsed.
pub fn parse_deplete_rrna_report(report_json: &str) -> Result<DepleteRrnaReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_deplete_rrna_report(report_json))
        .context("parse deplete rrna report")
}
