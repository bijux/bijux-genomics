//! Admit declared BAM alignment fixture truth against canonical corpus observations.
use super::validate_bam_corpus_fixture_manifest_path;
use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::Path;

mod admission;
mod model;
mod observations;
use admission::{validate_bundle_contract, validate_manifest_contract};
pub use model::BamAlignmentTruthValidationReport;
use model::{BamAlignmentSampleTruth, BamAlignmentTruthBundle, BamAlignmentTruthManifest};
use observations::{
    build_actual_truth_bundle, load_bam_alignment_truth_bundle,
    load_bam_alignment_truth_manifest_path,
};

pub const BAM_ALIGNMENT_TRUTH_FIXTURE_ID: &str = "bam-alignment-truth";
pub const BAM_ALIGNMENT_TRUTH_MANIFEST_SCHEMA_VERSION: &str = "bijux.bench.bam_alignment_truth.v1";
const BAM_ALIGNMENT_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.bam_alignment_truth.expected.v1";
const BAM_ALIGNMENT_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_alignment_truth.validation.v1";

/// Validate declared alignment truth without writing or running tools.
///
/// # Errors
/// Returns the existing source or truth-admission error before presenting a report.
pub fn validate_bam_alignment_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamAlignmentTruthValidationReport> {
    let manifest = load_bam_alignment_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("BAM alignment truth manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_path = resolve_manifest_relative_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("BAM alignment truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_bam_alignment_truth_bundle(&expected_path)?;
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
            "BAM alignment truth sample count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for sample_id in &manifest.sample_ids {
        let expected_sample = expected_map.get(sample_id.as_str()).ok_or_else(|| {
            anyhow!("expected BAM alignment truth is missing sample `{sample_id}`")
        })?;
        let actual_sample = actual_map.get(sample_id.as_str()).ok_or_else(|| {
            anyhow!("observed BAM alignment truth is missing sample `{sample_id}`")
        })?;
        if expected_sample != actual_sample {
            return Err(anyhow!("BAM alignment truth drifted for sample `{sample_id}`"));
        }
    }

    let checked_samples =
        actual.sample_truths.iter().map(|sample| sample.sample_id.clone()).collect::<Vec<_>>();
    let validated_record_count =
        actual.sample_truths.iter().map(|sample| sample.records.len()).sum::<usize>();

    Ok(BamAlignmentTruthValidationReport {
        schema_version: BAM_ALIGNMENT_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_sample_count: checked_samples.len(),
        validated_record_count,
        valid: true,
        checked_samples,
    })
}
