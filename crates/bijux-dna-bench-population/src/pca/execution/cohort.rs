//! Owner: bijux-dna-bench-population
//! PCA execution cohort boundary.
use crate::pca::admission::cohort::write_population_labels_manifest;
use crate::pca::model::DEFAULT_INPUT_POPULATION_LABELS_NAME;
use crate::pca::model::DEFAULT_INPUT_POPULATION_METADATA_NAME;
use crate::pca::model::DEFAULT_INPUT_SAMPLE_METADATA_NAME;
use crate::pca::model::DEFAULT_INPUT_VCF_NAME;
use anyhow::Context;
use anyhow::Result;
use std::path::Path;

pub(crate) struct StagedCohort {
    pub(crate) stage_root: std::path::PathBuf,
    pub(crate) input_vcf_path: std::path::PathBuf,
    pub(crate) population_labels_manifest_path: std::path::PathBuf,
}

pub(crate) fn stage_cohort(
    cohort: &super::super::admission::input::CohortAdmission,
    output_root: &Path,
) -> Result<StagedCohort> {
    let super::super::admission::input::CohortAdmission {
        input_vcf_source,
        sample_metadata_source,
        population_metadata_source,
        expected_samples,
        metadata_by_sample,
        ..
    } = cohort;
    let artifacts_root = output_root.join("artifacts");
    let input_root = artifacts_root.join("input");
    let stage_root = artifacts_root.join("stage");
    bijux_dna_infra::ensure_dir(&input_root)
        .with_context(|| format!("create {}", input_root.display()))?;
    bijux_dna_infra::ensure_dir(&stage_root)
        .with_context(|| format!("create {}", stage_root.display()))?;

    let input_vcf_path = input_root.join(DEFAULT_INPUT_VCF_NAME);
    bijux_dna_infra::copy_file(input_vcf_source, &input_vcf_path).with_context(|| {
        format!("copy {} to {}", input_vcf_source.display(), input_vcf_path.display())
    })?;
    let sample_metadata_path = input_root.join(DEFAULT_INPUT_SAMPLE_METADATA_NAME);
    bijux_dna_infra::copy_file(sample_metadata_source, &sample_metadata_path).with_context(
        || {
            format!(
                "copy {} to {}",
                sample_metadata_source.display(),
                sample_metadata_path.display()
            )
        },
    )?;
    let population_metadata_path = input_root.join(DEFAULT_INPUT_POPULATION_METADATA_NAME);
    bijux_dna_infra::copy_file(population_metadata_source, &population_metadata_path)
        .with_context(|| {
            format!(
                "copy {} to {}",
                population_metadata_source.display(),
                population_metadata_path.display()
            )
        })?;
    let population_labels_manifest_path = input_root.join(DEFAULT_INPUT_POPULATION_LABELS_NAME);
    write_population_labels_manifest(
        &population_labels_manifest_path,
        metadata_by_sample,
        expected_samples,
    )?;

    Ok(StagedCohort { stage_root, input_vcf_path, population_labels_manifest_path })
}
