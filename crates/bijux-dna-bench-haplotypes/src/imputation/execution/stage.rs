//! Native imputation execution with explicit truth and missingness policy.
use crate::imputation::model::GOVERNED_IMPUTE_SEED;
use crate::imputation::model::GOVERNED_IMPUTE_THREADS;
use crate::panel_contracts::GovernedVcfPanelWorkflowSmokeContract;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_impute_stage;
use bijux_dna_stages_vcf::pipeline::ImputationAcceptMode;
use bijux_dna_stages_vcf::pipeline::ImputeBackend;
use bijux_dna_stages_vcf::pipeline::ImputeStageOutputs;
use bijux_dna_stages_vcf::pipeline::ImputeStageParams;

pub(crate) fn execute_stage(
    prepared: &super::inputs::PreparedImputationInputs,
    contract: &GovernedVcfPanelWorkflowSmokeContract,
) -> Result<ImputeStageOutputs> {
    let stage_outputs = run_impute_stage(
        &prepared.input_vcf_path,
        &prepared.stage_root,
        &prepared.species_context,
        &ImputeStageParams {
            species_id: prepared.species_context.species_id.clone(),
            build_id: prepared.species_context.build_id.clone(),
            backend: prepared.backend,
            panel_id: Some(contract.panel.clone()),
            map_id: (!matches!(prepared.backend, ImputeBackend::Beagle))
                .then(|| contract.map.clone()),
            threads: GOVERNED_IMPUTE_THREADS,
            seed: GOVERNED_IMPUTE_SEED,
            emit_ds: true,
            emit_gp: true,
            truth_vcf: Some(prepared.truth_vcf_path.clone()),
            imputation_accept_mode: ImputationAcceptMode::MarkNonProduction,
            chunk_window_bp: None,
            chunk_overlap_bp: 0,
        },
    )
    .with_context(|| {
        format!("run governed VCF impute smoke from {}", prepared.input_vcf_path.display())
    })?;

    Ok(stage_outputs)
}
