//! Owner: bijux-dna-bench-population
//! Population structure execution evidence ownership.
use crate::population_structure::execution::artifacts::ensure_file_exists;
use crate::population_structure::execution::artifacts::read_json;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_ADMIXTURE_REPORT_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_LOGS_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_PCA_REPORT_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_PRUNED_VARIANTS_NAME;
use crate::population_structure::model::DEFAULT_OUTPUT_SOURCE_STAGE_REPORT_NAME;
use crate::population_structure::publication::rebase_population_structure_paths;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) struct CollectedPopulationEvidence {
    pub(crate) stage: serde_json::Value,
    pub(crate) pca: std::path::PathBuf,
    pub(crate) admixture: std::path::PathBuf,
}

pub(crate) fn collect_evidence(
    repo_root: &Path,
    upstream: &crate::population_structure::admission::upstream::ConsumedPopulationInputs,
    layout: &crate::population_structure::execution::layout::PopulationArtifactLayout,
    output_root: &Path,
    stage_outputs: &bijux_dna_stages_vcf::pipeline::PopulationStructureStageOutputs,
) -> Result<CollectedPopulationEvidence> {
    let crate::population_structure::admission::upstream::ConsumedPopulationInputs {
        pca_report,
        admixture_report,
        upstream_population_labels_manifest,
        ..
    } = upstream;
    let crate::population_structure::execution::layout::PopulationArtifactLayout {
        published_stage_root,
        ..
    } = layout;
    let source_population_structure_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_STAGE_REPORT_NAME);
    let mut source_population_structure_json = read_json(&stage_outputs.population_structure_json)?;
    rebase_population_structure_paths(
        &mut source_population_structure_json,
        repo_root,
        published_stage_root,
        upstream_population_labels_manifest,
    )?;
    bijux_dna_infra::atomic_write_json(
        &stage_outputs.population_structure_json,
        &source_population_structure_json,
    )?;
    bijux_dna_infra::atomic_write_json(
        &source_population_structure_path,
        &source_population_structure_json,
    )?;
    let source_pruned_variants_path = output_root.join(DEFAULT_OUTPUT_SOURCE_PRUNED_VARIANTS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.pruned_variants_tsv, &source_pruned_variants_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                stage_outputs.pruned_variants_tsv.display(),
                source_pruned_variants_path.display()
            )
        })?;
    let source_logs_path = output_root.join(DEFAULT_OUTPUT_SOURCE_LOGS_NAME);
    bijux_dna_infra::copy_file(&stage_outputs.logs_txt, &source_logs_path).with_context(|| {
        format!("copy {} to {}", stage_outputs.logs_txt.display(), source_logs_path.display())
    })?;

    let source_pca_report_source = repo_root.join(&pca_report.pca_json_path);
    let source_admixture_report_source = repo_root.join(&admixture_report.admixture_json_path);
    ensure_file_exists(&source_pca_report_source, "consumed PCA report")?;
    ensure_file_exists(&source_admixture_report_source, "consumed admixture report")?;

    let source_pca_report_path = output_root.join(DEFAULT_OUTPUT_SOURCE_PCA_REPORT_NAME);
    bijux_dna_infra::copy_file(&source_pca_report_source, &source_pca_report_path).with_context(
        || {
            format!(
                "copy {} to {}",
                source_pca_report_source.display(),
                source_pca_report_path.display()
            )
        },
    )?;
    let source_admixture_report_path =
        output_root.join(DEFAULT_OUTPUT_SOURCE_ADMIXTURE_REPORT_NAME);
    bijux_dna_infra::copy_file(&source_admixture_report_source, &source_admixture_report_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                source_admixture_report_source.display(),
                source_admixture_report_path.display()
            )
        })?;

    Ok(CollectedPopulationEvidence {
        stage: source_population_structure_json,
        pca: source_pca_report_path,
        admixture: source_admixture_report_path,
    })
}
