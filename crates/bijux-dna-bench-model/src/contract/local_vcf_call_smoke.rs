//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted local VCF calling smoke evidence.

use anyhow::{bail, Result};

use crate::model::local_vcf_call_smoke::{
    BenchmarkLocalVcfCallSmokeMetrics, BenchmarkLocalVcfCallSmokeReport,
    BENCHMARK_LOCAL_VCF_CALL_SMOKE_METRICS_SCHEMA_V1, BENCHMARK_LOCAL_VCF_CALL_SMOKE_SCHEMA_V1,
};

const STAGE_ID: &str = "vcf.call";
const RESOLVED_STAGE_ID: &str = "vcf.call_diploid";

struct CallEvidenceState<'a> {
    stage_id: &'a str,
    resolved_stage_id: &'a str,
    tool_id: &'a str,
    variant_count: u64,
    snp_count: u64,
    indel_count: u64,
    sample_count: u64,
    exit_code: i32,
}

/// Validates a persisted local VCF calling metrics record.
///
/// # Errors
///
/// Returns an error when its schema, identities, inputs, counts, or exit state are invalid.
pub fn validate_benchmark_local_vcf_call_smoke_metrics(
    metrics: &BenchmarkLocalVcfCallSmokeMetrics,
) -> Result<()> {
    if metrics.schema_version != BENCHMARK_LOCAL_VCF_CALL_SMOKE_METRICS_SCHEMA_V1 {
        bail!("unsupported local VCF call smoke metrics schema");
    }
    validate_identities_and_counts(&CallEvidenceState {
        stage_id: &metrics.stage_id,
        resolved_stage_id: &metrics.resolved_stage_id,
        tool_id: &metrics.tool_id,
        variant_count: metrics.variant_count,
        snp_count: metrics.snp_count,
        indel_count: metrics.indel_count,
        sample_count: metrics.sample_count,
        exit_code: metrics.exit_code,
    })?;
    for (name, value) in [
        ("corpus_id", metrics.corpus_id.as_str()),
        ("sample_id", metrics.sample_id.as_str()),
        ("input_bam", metrics.input_bam.as_str()),
        ("reference", metrics.reference.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!("local VCF call smoke metrics have empty {name}");
        }
    }
    Ok(())
}

/// Validates a persisted local VCF calling smoke report.
///
/// # Errors
///
/// Returns an error when identities, paths, timing, counts, validation checks, or output field
/// availability do not satisfy the successful calling contract.
pub fn validate_benchmark_local_vcf_call_smoke_report(
    report: &BenchmarkLocalVcfCallSmokeReport,
) -> Result<()> {
    if report.schema_version != BENCHMARK_LOCAL_VCF_CALL_SMOKE_SCHEMA_V1 {
        bail!("unsupported local VCF call smoke report schema");
    }
    validate_identities_and_counts(&CallEvidenceState {
        stage_id: &report.stage_id,
        resolved_stage_id: &report.resolved_stage_id,
        tool_id: &report.tool_id,
        variant_count: report.variant_count,
        snp_count: report.snp_count,
        indel_count: report.indel_count,
        sample_count: report.sample_count,
        exit_code: report.exit_code,
    })?;
    for (name, value) in [
        ("command", report.command.as_str()),
        ("corpus_id", report.corpus_id.as_str()),
        ("sample_id", report.sample_id.as_str()),
        ("sample_name", report.sample_name.as_str()),
        ("input_bam", report.input_bam.as_str()),
        ("input_bam_index", report.input_bam_index.as_str()),
        ("reference", report.reference.as_str()),
        ("materialized_reference_path", report.materialized_reference_path.as_str()),
        ("output_root", report.output_root.as_str()),
        ("output_vcf_path", report.output_vcf_path.as_str()),
        ("output_tbi_path", report.output_tbi_path.as_str()),
        ("metrics_path", report.metrics_path.as_str()),
        ("stage_result_manifest_path", report.stage_result_manifest_path.as_str()),
        ("started_at", report.started_at.as_str()),
        ("finished_at", report.finished_at.as_str()),
    ] {
        if value.trim().is_empty() {
            bail!("local VCF call smoke report has empty {name}");
        }
    }
    if !report.command.ends_with(&format!("--tool-id {}", report.tool_id)) {
        bail!("local VCF call smoke command does not bind the reported tool");
    }
    if !report.elapsed_seconds.is_finite() || report.elapsed_seconds < 0.0 {
        bail!("local VCF call smoke report has invalid elapsed time");
    }
    if !report.parseable || !report.gt_present {
        bail!("local VCF call smoke output is not parseable with the required GT field");
    }
    for check in ["bgzip", "tabix_index", "sorted", "contig_header_sane"] {
        if report.validation_checks.get(check) != Some(&true) {
            bail!("local VCF call smoke report failed required validation check `{check}`");
        }
    }
    if report.validation_checks.values().any(|passed| !passed) {
        bail!("local VCF call smoke report contains a failed validation check");
    }
    Ok(())
}

fn validate_identities_and_counts(state: &CallEvidenceState<'_>) -> Result<()> {
    if state.stage_id != STAGE_ID || state.resolved_stage_id != RESOLVED_STAGE_ID {
        bail!("local VCF call smoke stage identity drifted");
    }
    if state.tool_id.trim().is_empty() {
        bail!("local VCF call smoke has an empty tool identity");
    }
    if state.exit_code != 0 || state.sample_count == 0 {
        bail!("successful local VCF call smoke evidence requires samples and exit code zero");
    }
    if state.snp_count.checked_add(state.indel_count) != Some(state.variant_count) {
        bail!("local VCF call smoke variant counts do not close");
    }
    Ok(())
}
