//! Execute the native VCF stage with explicit scientific parameters.
use crate::qc::execution::inputs::PreparedInputs;
use crate::qc::model::GovernedVcfQcSmokeContract;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_qc_stage;
use bijux_dna_stages_vcf::pipeline::QcStageOutputs;
use bijux_dna_stages_vcf::pipeline::QcStageParams;

pub(crate) fn execute_stage(
    prepared: &PreparedInputs,
    contract: &GovernedVcfQcSmokeContract,
) -> Result<QcStageOutputs> {
    let input_vcf = &prepared.input_vcf;
    let stage_root = &prepared.stage_root;
    let stage_outputs = run_qc_stage(
        input_vcf,
        stage_root,
        &QcStageParams {
            sample_name: contract.sample_name.clone(),
            is_ancient_dna: false,
            allow_hwe_for_ancient: false,
            production_profile: false,
            pre_filter_vcf: None,
        },
    )
    .with_context(|| format!("run governed VCF QC smoke from {}", input_vcf.display()))?;

    Ok(stage_outputs)
}
