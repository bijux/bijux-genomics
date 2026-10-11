//! Validate observed genotype semantics and construct persisted scientific metrics.
use crate::bam::execution::native::CalledArtifacts;
use crate::bam::model::GovernedVcfCallBamSmokeContract;
use crate::call_gl::observations::variants::summarize_likelihood_fields;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_model::validate_benchmark_local_vcf_call_gl_smoke_metrics;
use bijux_dna_bench_model::BenchmarkLocalVcfCallGlSmokeMetrics as LocalVcfCallGlSmokeMetrics;
use bijux_dna_bench_model::BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_V1;
use bijux_dna_stages_vcf::vcf_io::vcf_sample_count as parse_output_sample_count;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use bijux_dna_stages_vcf::vcf_io::VcfValidationSummary;

pub(crate) struct StageEvidence {
    pub(crate) metrics: LocalVcfCallGlSmokeMetrics,
    pub(crate) validation: VcfValidationSummary,
}

pub(crate) fn observe(
    contract: &GovernedVcfCallBamSmokeContract,
    artifacts: &CalledArtifacts,
) -> Result<StageEvidence> {
    let output_vcf = &artifacts.vcf;
    let validation =
        vcf_validate_input(output_vcf, VcfFieldRequirement { require_gt: false, require_gl: true })
            .with_context(|| format!("validate {}", output_vcf.display()))?;
    let likelihood_summary = summarize_likelihood_fields(output_vcf)
        .with_context(|| format!("summarize likelihood fields in {}", output_vcf.display()))?;
    let sample_count = parse_output_sample_count(output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;

    let metrics = LocalVcfCallGlSmokeMetrics {
        schema_version: BENCHMARK_LOCAL_VCF_CALL_GL_SMOKE_METRICS_SCHEMA_V1.to_string(),
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        sample_id: contract.sample_id.clone(),
        input_bam: contract.input_bam.clone(),
        reference: contract.reference.clone(),
        likelihood_field: likelihood_summary.likelihood_field.clone(),
        sites_with_likelihoods: likelihood_summary.sites_with_likelihoods,
        samples_with_likelihoods: likelihood_summary.samples_with_likelihoods,
        missing_likelihoods: likelihood_summary.missing_likelihoods,
        sample_count,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };
    validate_benchmark_local_vcf_call_gl_smoke_metrics(&metrics)?;
    Ok(StageEvidence { metrics, validation })
}
