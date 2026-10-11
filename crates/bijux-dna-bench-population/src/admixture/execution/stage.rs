//! Owner: bijux-dna-bench-population
//! Execute the scientific stage using admitted bindings and prepared cohort paths.
use crate::admixture::model::GOVERNED_VCF_ADMIXTURE_K_VALUES;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_admixture_stage;
use bijux_dna_stages_vcf::pipeline::AdmixtureStageParams;

pub(crate) fn execute_stage(
    staged: &super::cohort::StagedCohort,
    contract: &crate::admixture::model::GovernedVcfAdmixtureSmokeContract,
) -> Result<bijux_dna_stages_vcf::pipeline::AdmixtureStageOutputs> {
    let super::cohort::StagedCohort { stage_root, input_vcf_path, population_labels_manifest_path } =
        staged;
    let stage_outputs = run_admixture_stage(
        input_vcf_path,
        stage_root,
        &AdmixtureStageParams {
            toolchain: contract.tool.clone(),
            k_values: GOVERNED_VCF_ADMIXTURE_K_VALUES.to_vec(),
            sample_metadata_manifest: Some(population_labels_manifest_path.clone()),
        },
    )
    .with_context(|| {
        format!("run governed VCF admixture smoke from {}", input_vcf_path.display())
    })?;

    Ok(stage_outputs)
}
