use anyhow::Result;
use bijux_dna_core::contract::{ExecutionEdge, ExecutionGraph};
use bijux_dna_core::ids::StepId;
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;
use bijux_dna_stage_contract::{execution_step_from_stage_plan, StagePlanV1};
use sha2::Digest;

use crate::api::VcfPipelineInputs;

fn short_species_context_digest(
    species_id: &str,
    build_id: &str,
    contig_set_digest: &str,
) -> String {
    let seed = format!("{species_id}|{build_id}|{contig_set_digest}");
    let mut hasher = sha2::Sha256::new();
    hasher.update(seed.as_bytes());
    let full = format!("{:x}", hasher.finalize());
    full.chars().take(12).collect()
}

/// # Errors
/// Returns an error if pipeline graph materialization fails.
pub fn build_vcf_pipeline_graph(
    inputs: &VcfPipelineInputs,
    resolved_coverage: CoverageRegime,
    plans: &[StagePlanV1],
) -> Result<ExecutionGraph> {
    let steps = plans
        .iter()
        .map(execution_step_from_stage_plan)
        .collect::<Vec<_>>();
    let edges = plans
        .windows(2)
        .map(|pair| {
            ExecutionEdge::new(
                StepId::new(pair[0].stage_id.to_string()),
                StepId::new(pair[1].stage_id.to_string()),
            )
        })
        .collect::<Vec<_>>();
    let flavor_base = match resolved_coverage {
        CoverageRegime::LowCovGl => "downstream_lowcov_gl",
        CoverageRegime::Diploid => "downstream_diploid",
        CoverageRegime::Pseudohaploid => "downstream_pseudohaploid",
    };
    let species_digest = short_species_context_digest(
        &inputs.species_context.species_id,
        &inputs.species_context.build_id,
        &inputs.species_context.contig_set_digest,
    );
    let flavor = format!("{flavor_base}_sctx_{species_digest}");
    Ok(ExecutionGraph::new(
        format!("vcf-to-vcf__{flavor}__v2"),
        crate::PLANNER_VERSION,
        inputs.policy,
        steps,
        edges,
    )?)
}
