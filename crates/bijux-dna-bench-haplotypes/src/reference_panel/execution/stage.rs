//! Execute reference panel preparation through the native stage owner.
use crate::reference_panel::model::GovernedVcfPrepareReferencePanelSmokeContract;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_prepare_reference_panel_stage;
use bijux_dna_stages_vcf::pipeline::PrepareReferencePanelOutputs;
use bijux_dna_stages_vcf::pipeline::PrepareReferencePanelParams;

pub(crate) fn execute_stage(
    prepared: &super::inputs::PreparedPanelInputs,
    contract: &GovernedVcfPrepareReferencePanelSmokeContract,
) -> Result<PrepareReferencePanelOutputs> {
    let stage_outputs = run_prepare_reference_panel_stage(
        &prepared.input_vcf_path,
        &prepared.raw_panel_path,
        &prepared.stage_root,
        &prepared.species_context,
        &PrepareReferencePanelParams {
            species_id: prepared.species_context.species_id.clone(),
            build_id: prepared.species_context.build_id.clone(),
            panel_id: Some(contract.panel.clone()),
        },
    )
    .with_context(|| {
        format!(
            "run governed VCF panel preparation smoke from {}",
            prepared.raw_panel_path.display()
        )
    })?;

    Ok(stage_outputs)
}
