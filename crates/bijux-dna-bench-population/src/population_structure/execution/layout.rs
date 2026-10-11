//! Owner: bijux-dna-bench-population
//! Population structure execution layout ownership.
use crate::population_structure::model::DEFAULT_OUTPUT_REPORT_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_ADMIXTURE_REPORT_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_PCA_REPORT_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_PRUNED_VARIANTS_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_STAGE_REPORT_NAME;
use crate::population_structure::model::DEFAULT_STAGE_RESULT_NAME;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) struct PopulationArtifactLayout {
    pub(crate) published_population_structure_json_path: std::path::PathBuf,
    pub(crate) published_stage_root: std::path::PathBuf,
    pub(crate) published_source_population_structure_path: std::path::PathBuf,
    pub(crate) published_source_pruned_variants_path: std::path::PathBuf,
    pub(crate) published_source_logs_path: std::path::PathBuf,
    pub(crate) published_source_pca_report_path: std::path::PathBuf,
    pub(crate) published_source_admixture_report_path: std::path::PathBuf,
    pub(crate) published_stage_result_manifest_path: std::path::PathBuf,
    pub(crate) stage_root: std::path::PathBuf,
}

pub(crate) fn prepare_layout(
    published_output_root: &Path,
    output_root: &Path,
) -> Result<PopulationArtifactLayout> {
    let published_population_structure_json_path =
        published_output_root.join(DEFAULT_OUTPUT_REPORT_NAME);
    let published_artifacts_root = published_output_root.join("artifacts");
    let published_stage_root = published_artifacts_root.join("stage");
    let published_source_population_structure_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_STAGE_REPORT_NAME);
    let published_source_pruned_variants_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_PRUNED_VARIANTS_NAME);
    let published_source_logs_path = published_output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    let published_source_pca_report_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_PCA_REPORT_NAME);
    let published_source_admixture_report_path =
        published_output_root.join(DEFAULT_OUTPUT_SOURCE_ADMIXTURE_REPORT_NAME);
    let published_stage_result_manifest_path =
        published_output_root.join(DEFAULT_STAGE_RESULT_NAME);
    let artifacts_root = output_root.join("artifacts");
    let stage_root = artifacts_root.join("stage");
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;

    Ok(PopulationArtifactLayout {
        published_population_structure_json_path,
        published_stage_root,
        published_source_population_structure_path,
        published_source_pruned_variants_path,
        published_source_logs_path,
        published_source_pca_report_path,
        published_source_admixture_report_path,
        published_stage_result_manifest_path,
        stage_root,
    })
}
