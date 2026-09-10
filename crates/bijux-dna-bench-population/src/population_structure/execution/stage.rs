//! Owner: bijux-dna-bench-population
//! Execute the population stage with explicit upstream and tool parameter contracts.
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::run_population_structure_stage;
use bijux_dna_stages_vcf::pipeline::AdmixtureStageParams;
use bijux_dna_stages_vcf::pipeline::PopulationPreprocessingParams;
use bijux_dna_stages_vcf::pipeline::PopulationStructureStageParams;

pub(crate) fn execute_stage(
    upstream: &super::super::admission::upstream::ConsumedPopulationInputs,
    layout: &super::layout::PopulationArtifactLayout,
    contract: &super::super::model::GovernedVcfPopulationStructureSmokeContract,
) -> Result<bijux_dna_stages_vcf::pipeline::PopulationStructureStageOutputs> {
    let super::super::admission::upstream::ConsumedPopulationInputs {
        upstream_input_vcf,
        upstream_population_labels_manifest,
        ..
    } = upstream;
    let super::layout::PopulationArtifactLayout { stage_root, .. } = layout;
    let stage_outputs = run_population_structure_stage(
        upstream_input_vcf,
        stage_root,
        &PopulationStructureStageParams {
            toolchain: contract.pca_tool_id.clone(),
            smartpca: contract.pca_tool_id == "eigensoft",
            run_admixture: true,
            sample_metadata_manifest: Some(upstream_population_labels_manifest.clone()),
            admixture_params: Some(AdmixtureStageParams {
                toolchain: contract.admixture_tool_id.clone(),
                k_values: vec![2, 3, 4],
                sample_metadata_manifest: Some(upstream_population_labels_manifest.clone()),
            }),
            preprocessing: PopulationPreprocessingParams::default(),
        },
    )
    .with_context(|| {
        format!("run governed VCF population-structure smoke from {}", upstream_input_vcf.display())
    })?;

    Ok(stage_outputs)
}
