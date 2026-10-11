//! Native phasing execution from admitted fixture inputs.
use crate::panel_contracts::GovernedVcfPanelWorkflowSmokeContract;
use crate::phasing::model::GOVERNED_PHASING_REGION;
use crate::phasing::model::GOVERNED_PHASING_SEED;
use crate::phasing::model::GOVERNED_PHASING_THREADS;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_phasing_stage;
use bijux_dna_stages_vcf::pipeline::PhasingStageOutputs;
use bijux_dna_stages_vcf::pipeline::PhasingStageParams;

pub(crate) fn execute_stage(
    prepared: &super::inputs::PreparedPhasingInputs,
    contract: &GovernedVcfPanelWorkflowSmokeContract,
) -> Result<PhasingStageOutputs> {
    let stage_outputs = run_phasing_stage(
        &prepared.input_vcf_path,
        &prepared.stage_root,
        &prepared.species_context,
        &PhasingStageParams {
            species_id: prepared.species_context.species_id.clone(),
            build_id: prepared.species_context.build_id.clone(),
            backend: prepared.backend,
            panel_id: Some(contract.panel.clone()),
            map_id: Some(contract.map.clone()),
            threads: GOVERNED_PHASING_THREADS,
            seed: GOVERNED_PHASING_SEED,
            region: Some(GOVERNED_PHASING_REGION.to_string()),
            allow_gl_only_input: false,
        },
    )
    .with_context(|| {
        format!("run governed VCF phasing smoke from {}", prepared.input_vcf_path.display())
    })?;

    Ok(stage_outputs)
}
