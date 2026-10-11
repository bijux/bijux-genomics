//! Owner: bijux-dna-bench-population
//! Execute the scientific stage using admitted bindings and prepared cohort paths.
use crate::pca::model::GOVERNED_PCA_COMPONENT_COUNT;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_pca_stage;
use bijux_dna_stages_vcf::pipeline::PcaStageParams;
use bijux_dna_stages_vcf::pipeline::PopulationPreprocessingParams;

pub(crate) fn execute_stage(
    staged: &super::cohort::StagedCohort,
    contract: &crate::pca::model::GovernedVcfPcaSmokeContract,
) -> Result<bijux_dna_stages_vcf::pipeline::PcaStageOutputs> {
    let super::cohort::StagedCohort { stage_root, input_vcf_path, population_labels_manifest_path } =
        staged;
    let stage_outputs = run_pca_stage(
        input_vcf_path,
        stage_root,
        &PcaStageParams {
            toolchain: contract.tool.clone(),
            components: GOVERNED_PCA_COMPONENT_COUNT,
            sample_metadata_manifest: Some(population_labels_manifest_path.clone()),
            preprocessing: PopulationPreprocessingParams::default(),
        },
    )
    .with_context(|| format!("run governed VCF PCA smoke from {}", input_vcf_path.display()))?;

    Ok(stage_outputs)
}
