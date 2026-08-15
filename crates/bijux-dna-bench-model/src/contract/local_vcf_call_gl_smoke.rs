//! Owner: bijux-dna-bench-model
//! Semantic admission for persisted local VCF genotype-likelihood calling smoke evidence.

use anyhow::{bail, Result};

use crate::model::local_vcf_call_gl_smoke::{
    BenchmarkLocalVcfCallGlSmokeMetrics, BenchmarkLocalVcfCallGlSmokeReport,
    BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_V1,
    BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_SCHEMA_V1,
};

const STAGE_ID: &str = "vcf.call_gl";
const LIKELIHOOD_FIELDS: [&str; 3] = ["GL", "GP", "PL"];

struct CallGlEvidenceState<'a> {
    stage_id: &'a str,
    tool_id: &'a str,
    likelihood_field: &'a str,
    sites_with_likelihoods: u64,
    samples_with_likelihoods: u64,
    sample_count: u64,
    exit_code: i32,
}

/// Validates a persisted local VCF genotype-likelihood calling metrics record.
///
/// # Errors
///
/// Returns an error when its schema, identities, inputs, likelihood evidence, or exit state are
/// invalid.
pub fn validate_benchmark_local_vcf_call_gl_smoke_metrics(
    metrics: &BenchmarkLocalVcfCallGlSmokeMetrics,
) -> Result<()> {
    if metrics.schema_version != BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_V1 {
        bail!("unsupported local VCF call GL smoke metrics schema");
    }
    validate_identities_and_likelihoods(&CallGlEvidenceState {
        stage_id: &metrics.stage_id,
        tool_id: &metrics.tool_id,
        likelihood_field: &metrics.likelihood_field,
        sites_with_likelihoods: metrics.sites_with_likelihoods,
        samples_with_likelihoods: metrics.samples_with_likelihoods,
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
            bail!("local VCF call GL smoke metrics have empty {name}");
        }
    }
    Ok(())
}

/// Validates a persisted local VCF genotype-likelihood calling report.
///
/// # Errors
///
/// Returns an error when identities, paths, timing, likelihood evidence, validation checks, or
/// output field availability do not satisfy the successful calling contract.
pub fn validate_benchmark_local_vcf_call_gl_smoke_report(
    report: &BenchmarkLocalVcfCallGlSmokeReport,
) -> Result<()> {
    if report.schema_version != BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_SCHEMA_V1 {
        bail!("unsupported local VCF call GL smoke report schema");
    }
    validate_identities_and_likelihoods(&CallGlEvidenceState {
        stage_id: &report.stage_id,
        tool_id: &report.tool_id,
        likelihood_field: &report.likelihood_field,
        sites_with_likelihoods: report.sites_with_likelihoods,
        samples_with_likelihoods: report.samples_with_likelihoods,
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
            bail!("local VCF call GL smoke report has empty {name}");
        }
    }
    if !report.command.ends_with(&format!("--tool-id {}", report.tool_id)) {
        bail!("local VCF call GL smoke command does not bind the reported tool");
    }
    if !report.elapsed_seconds.is_finite() || report.elapsed_seconds < 0.0 {
        bail!("local VCF call GL smoke report has invalid elapsed time");
    }
    if !report.parseable || !report.gl_present {
        bail!("local VCF call GL smoke output is not parseable with a likelihood field");
    }
    for check in ["bgzip", "tabix_index", "sorted", "contig_header_sane"] {
        if report.validation_checks.get(check) != Some(&true) {
            bail!("local VCF call GL smoke report failed required validation check `{check}`");
        }
    }
    if report.validation_checks.values().any(|passed| !passed) {
        bail!("local VCF call GL smoke report contains a failed validation check");
    }
    Ok(())
}

fn validate_identities_and_likelihoods(state: &CallGlEvidenceState<'_>) -> Result<()> {
    if state.stage_id != STAGE_ID {
        bail!("local VCF call GL smoke stage identity drifted");
    }
    if state.tool_id.trim().is_empty() {
        bail!("local VCF call GL smoke has an empty tool identity");
    }
    if !LIKELIHOOD_FIELDS.contains(&state.likelihood_field) {
        bail!("local VCF call GL smoke has an unsupported likelihood field");
    }
    if state.exit_code != 0
        || state.sample_count == 0
        || state.sites_with_likelihoods == 0
        || state.samples_with_likelihoods == 0
        || state.samples_with_likelihoods > state.sample_count
    {
        bail!("successful local VCF call GL smoke evidence requires likelihood-bearing samples and exit code zero");
    }
    Ok(())
}
