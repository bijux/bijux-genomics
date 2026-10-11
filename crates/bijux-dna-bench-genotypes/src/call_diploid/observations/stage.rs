//! Validate observed genotype semantics and construct persisted scientific metrics.
use crate::bam::execution::native::CalledArtifacts;
use crate::bam::model::GovernedVcfCallBamSmokeContract;
use crate::call_diploid::model::LocalVcfCallDiploidSmokeMetrics;
use crate::call_diploid::model::LOCAL_VCF_CALL_DIPLOID_SMOKE_METRICS_SCHEMA_VERSION;
use crate::call_diploid::observations::variants::summarize_diploid_genotypes;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::vcf_sample_count as parse_output_sample_count;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use bijux_dna_stages_vcf::vcf_io::VcfValidationSummary;

pub(crate) struct StageEvidence {
    pub(crate) metrics: LocalVcfCallDiploidSmokeMetrics,
    pub(crate) validation: VcfValidationSummary,
}

pub(crate) fn observe(
    contract: &GovernedVcfCallBamSmokeContract,
    artifacts: &CalledArtifacts,
) -> Result<StageEvidence> {
    let output_vcf = &artifacts.vcf;
    let validation =
        vcf_validate_input(output_vcf, VcfFieldRequirement { require_gt: true, require_gl: false })
            .with_context(|| format!("validate {}", output_vcf.display()))?;
    let genotype_summary = summarize_diploid_genotypes(output_vcf)
        .with_context(|| format!("summarize diploid genotypes in {}", output_vcf.display()))?;
    let sample_count = parse_output_sample_count(output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;

    let metrics = LocalVcfCallDiploidSmokeMetrics {
        schema_version: LOCAL_VCF_CALL_DIPLOID_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        sample_id: contract.sample_id.clone(),
        input_bam: contract.input_bam.clone(),
        reference: contract.reference.clone(),
        ploidy: "diploid",
        called_genotypes: genotype_summary.called_genotypes,
        heterozygous_count: genotype_summary.heterozygous_count,
        homozygous_ref_count: genotype_summary.homozygous_ref_count,
        homozygous_alt_count: genotype_summary.homozygous_alt_count,
        missing_count: genotype_summary.missing_count,
        sample_count,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };
    Ok(StageEvidence { metrics, validation })
}
