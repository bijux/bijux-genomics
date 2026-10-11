//! Validate observed genotype semantics and construct persisted scientific metrics.
use crate::bam::execution::native::CalledArtifacts;
use crate::call::model::GovernedVcfCallSmokeContract;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_bench_model::validate_benchmark_local_vcf_call_smoke_metrics;
use bijux_dna_bench_model::BenchmarkLocalVcfCallSmokeMetrics as LocalVcfCallSmokeMetrics;
use bijux_dna_bench_model::BENCHMARK_LOCAL_VCF_CALL_SMOKE_METRICS_SCHEMA_V1;
use bijux_dna_stages_vcf::metrics::parse_vcf_call_summary;
use bijux_dna_stages_vcf::vcf_io::vcf_sample_count as parse_output_sample_count;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use bijux_dna_stages_vcf::vcf_io::VcfValidationSummary;

pub(crate) struct StageEvidence {
    pub(crate) metrics: LocalVcfCallSmokeMetrics,
    pub(crate) validation: VcfValidationSummary,
}

pub(crate) fn observe(
    contract: &GovernedVcfCallSmokeContract,
    artifacts: &CalledArtifacts,
) -> Result<StageEvidence> {
    let output_vcf = &artifacts.vcf;
    let validation =
        vcf_validate_input(output_vcf, VcfFieldRequirement { require_gt: true, require_gl: false })
            .with_context(|| format!("validate {}", output_vcf.display()))?;
    let call_summary = parse_vcf_call_summary(output_vcf, &contract.sample_name)
        .with_context(|| format!("parse call summary from {}", output_vcf.display()))?;
    let sample_count = parse_output_sample_count(output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;

    let metrics = LocalVcfCallSmokeMetrics {
        schema_version: BENCHMARK_LOCAL_VCF_CALL_SMOKE_METRICS_SCHEMA_V1.to_string(),
        stage_id: contract.stage_id.clone(),
        resolved_stage_id: contract.resolved_stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        sample_id: contract.sample_id.clone(),
        input_bam: contract.input_bam.clone(),
        reference: contract.reference.clone(),
        variant_count: call_summary.variants_called,
        snp_count: call_summary.snps,
        indel_count: call_summary.indels,
        sample_count,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };
    validate_benchmark_local_vcf_call_smoke_metrics(&metrics)?;
    Ok(StageEvidence { metrics, validation })
}
