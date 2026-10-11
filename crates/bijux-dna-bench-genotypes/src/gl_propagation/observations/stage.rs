//! Admit likelihood field retention and sample cardinality before publication.
use crate::gl_propagation::execution::PreparedInputs;
use crate::gl_propagation::execution::StageArtifacts;
use crate::gl_propagation::model::GovernedVcfGlPropagationSmokeContract;
use crate::gl_propagation::model::LocalVcfGlPropagationSmokeMetrics;
use crate::gl_propagation::model::LOCAL_VCF_GL_PROPAGATION_SMOKE_METRICS_SCHEMA_VERSION;
use crate::gl_propagation::observations::variants::summarize_likelihood_fields;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::vcf_sample_count as parse_output_sample_count;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use bijux_dna_stages_vcf::vcf_io::VcfValidationSummary;

pub(crate) struct StageEvidence {
    pub(crate) metrics: LocalVcfGlPropagationSmokeMetrics,
    pub(crate) validation: VcfValidationSummary,
}
pub(crate) fn observe(
    contract: &GovernedVcfGlPropagationSmokeContract,
    prepared: &PreparedInputs,
    artifacts: &StageArtifacts,
) -> Result<StageEvidence> {
    let output_vcf = &artifacts.vcf;
    let input_summary = &prepared.summary;
    let validation =
        vcf_validate_input(output_vcf, VcfFieldRequirement { require_gt: true, require_gl: true })
            .with_context(|| format!("validate {}", output_vcf.display()))?;
    let output_summary = summarize_likelihood_fields(output_vcf)?;
    let lost_fields =
        input_summary.fields.difference(&output_summary.fields).cloned().collect::<Vec<_>>();
    if !lost_fields.is_empty() {
        bail!("governed gl_propagation smoke lost likelihood fields: {}", lost_fields.join(","));
    }
    let sample_count = parse_output_sample_count(output_vcf)
        .with_context(|| format!("count samples in {}", output_vcf.display()))?;
    if sample_count != 1 {
        bail!("governed gl_propagation smoke expects exactly one sample, found {sample_count}");
    }

    let metrics = LocalVcfGlPropagationSmokeMetrics {
        schema_version: LOCAL_VCF_GL_PROPAGATION_SMOKE_METRICS_SCHEMA_VERSION,
        stage_id: contract.stage_id.clone(),
        corpus_id: contract.corpus_id.clone(),
        input_fixture_id: contract.input_fixture_id.clone(),
        input_likelihood_fields: input_summary.fields.iter().cloned().collect(),
        output_likelihood_fields: output_summary.fields.iter().cloned().collect(),
        lost_fields,
        site_count_before: input_summary.site_count,
        site_count_after: output_summary.site_count,
        sample_count,
        tool_id: contract.tool_id.clone(),
        exit_code: 0,
    };
    Ok(StageEvidence { metrics, validation })
}
