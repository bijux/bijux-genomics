//! Compare declared duplicate and insert-size truth with existing corpus observations.
use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path as resolve_fixture_path};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::Path;
mod admission;
mod model;
mod observations;
use admission::{validate_bundle_contract, validate_manifest_contract};
pub use model::BamDuplicateInsertTruthValidationReport;
use observations::{
    build_actual_truth_bundle, load_bam_duplicate_insert_truth_bundle,
    load_bam_duplicate_insert_truth_manifest_path,
};

pub const BAM_DUPLICATE_INSERT_TRUTH_FIXTURE_ID: &str = "bam-duplicate-insert-truth";
pub const BAM_DUPLICATE_INSERT_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.bam_duplicate_insert_truth.v1";
const BAM_DUPLICATE_INSERT_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.bam_duplicate_insert_truth.expected.v1";
const BAM_DUPLICATE_INSERT_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_duplicate_insert_truth.validation.v1";

/// Validate governed duplicate-family and insert-size fixture truth.
///
/// # Errors
/// Returns the declaration, expected bundle or corpus observation error.
pub fn validate_bam_duplicate_insert_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamDuplicateInsertTruthValidationReport> {
    let manifest = load_bam_duplicate_insert_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM duplicate/insert truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!(
            "BAM duplicate/insert truth bundle is missing: {}",
            expected_path.display()
        ));
    }

    let expected = load_bam_duplicate_insert_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    let expected_map = expected
        .sample_truths
        .iter()
        .map(|sample| (sample.sample_id.as_str(), sample))
        .collect::<BTreeMap<_, _>>();
    let actual_map = actual
        .sample_truths
        .iter()
        .map(|sample| (sample.sample_id.as_str(), sample))
        .collect::<BTreeMap<_, _>>();
    if expected_map.len() != actual_map.len() {
        return Err(anyhow!(
            "BAM duplicate/insert truth sample count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for sample_id in &manifest.sample_ids {
        let expected_sample = expected_map.get(sample_id.as_str()).ok_or_else(|| {
            anyhow!("expected BAM duplicate/insert truth is missing sample `{sample_id}`")
        })?;
        let actual_sample = actual_map.get(sample_id.as_str()).ok_or_else(|| {
            anyhow!("observed BAM duplicate/insert truth is missing sample `{sample_id}`")
        })?;
        if expected_sample != actual_sample {
            return Err(anyhow!("BAM duplicate/insert truth drifted for sample `{sample_id}`"));
        }
    }

    let checked_samples =
        actual.sample_truths.iter().map(|sample| sample.sample_id.clone()).collect::<Vec<_>>();
    let validated_duplicate_family_bin_count = actual
        .sample_truths
        .iter()
        .map(|sample| sample.duplicate_family_histogram.len())
        .sum::<usize>();
    let validated_insert_size_bin_count =
        actual.sample_truths.iter().map(|sample| sample.insert_size_histogram.len()).sum::<usize>();

    Ok(BamDuplicateInsertTruthValidationReport {
        schema_version: BAM_DUPLICATE_INSERT_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_sample_count: checked_samples.len(),
        validated_duplicate_family_bin_count,
        validated_insert_size_bin_count,
        valid: true,
        checked_samples,
    })
}
