use anyhow::{anyhow, Context, Result};
use serde::Deserialize;

use super::super::{DepleteHostReportV1, PairedMode};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyDepleteHostReportV1 {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    host_fraction_removed: f64,
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    runtime_s: Option<f64>,
    memory_mb: Option<f64>,
}

fn parse_legacy_deplete_host_report(report_json: &str) -> Result<DepleteHostReportV1> {
    let legacy: LegacyDepleteHostReportV1 =
        serde_json::from_str(report_json).context("parse legacy deplete host report")?;
    if legacy.schema_version != "bijux.fastq.deplete_host.report.v1" {
        return Err(anyhow!("unsupported deplete host report schema {}", legacy.schema_version));
    }
    Ok(DepleteHostReportV1 {
        schema_version: "bijux.fastq.deplete_host.report.v2".to_string(),
        stage: legacy.stage_id.clone(),
        stage_id: legacy.stage_id,
        tool_id: legacy.tool_id,
        paired_mode: PairedMode::SingleEnd,
        threads: 1,
        reference_scope: crate::params::screen::ReferenceScope::Host,
        reference_catalog_id: "host_reference".to_string(),
        reference_index_artifact_id: "reference_index".to_string(),
        reference_index_backend: "bowtie2_build".to_string(),
        reference_build_id: None,
        reference_digest: None,
        masking_policy: crate::params::screen::ReferenceMaskingPolicy::Unmasked,
        decoy_policy: crate::params::screen::ReferenceDecoyPolicy::None,
        decoy_catalog_id: None,
        identity_threshold: 0.95,
        retained_read_policy: crate::params::screen::ReadRetentionPolicy::KeepNonHostReads,
        emit_removed_reads: true,
        report_format: crate::params::screen::MappingReportFormat::Bowtie2MetricsFile,
        retain_unmapped_pairs: false,
        input_r1: String::new(),
        input_r2: None,
        output_r1: String::new(),
        output_r2: None,
        removed_host_r1: String::new(),
        removed_host_r2: None,
        report_json: "host_depletion_report.json".to_string(),
        reads_in: legacy.reads_in,
        reads_out: legacy.reads_out,
        reads_removed: legacy.reads_in.saturating_sub(legacy.reads_out),
        bases_in: legacy.bases_in,
        bases_out: legacy.bases_out,
        bases_removed: legacy.bases_in.saturating_sub(legacy.bases_out),
        pairs_in: None,
        pairs_out: None,
        host_fraction_removed: legacy.host_fraction_removed,
        runtime_s: legacy.runtime_s,
        memory_mb: legacy.memory_mb,
        exit_code: None,
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: None,
    })
}

/// # Errors
/// Returns an error if the governed host-depletion report JSON cannot be parsed.
pub fn parse_deplete_host_report(report_json: &str) -> Result<DepleteHostReportV1> {
    serde_json::from_str(report_json)
        .or_else(|_| parse_legacy_deplete_host_report(report_json))
        .context("parse deplete host report")
}
