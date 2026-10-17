//! Compare declared endogenous-count truth with canonical corpus observations.
use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path as resolve_fixture_path};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::Path;
mod admission;
mod model;
mod observations;
use admission::{validate_bundle_contract, validate_manifest_contract};
pub use model::BamEndogenousTruthValidationReport;
use observations::{
    build_actual_truth_bundle, load_bam_endogenous_truth_bundle,
    load_bam_endogenous_truth_manifest_path,
};

pub const BAM_ENDOGENOUS_TRUTH_FIXTURE_ID: &str = "endogenous-truth";
pub const BAM_ENDOGENOUS_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.bam_endogenous_truth.v1";
const BAM_ENDOGENOUS_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.bam_endogenous_truth.expected.v1";
const BAM_ENDOGENOUS_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_endogenous_truth.validation.v1";

/// Validate the governed endogenous-count fixture truth.
///
/// # Errors
/// Returns the existing declaration, expected-record or local observation error.
pub fn validate_bam_endogenous_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamEndogenousTruthValidationReport> {
    let manifest = load_bam_endogenous_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM endogenous truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("BAM endogenous truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_bam_endogenous_truth_bundle(&expected_path)?;
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
            "BAM endogenous truth sample count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for case in &manifest.cases {
        let expected_sample = expected_map.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!("expected BAM endogenous truth is missing sample `{}`", case.sample_id)
        })?;
        let actual_sample = actual_map.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!("observed BAM endogenous truth is missing sample `{}`", case.sample_id)
        })?;
        if expected_sample != actual_sample {
            return Err(anyhow!("BAM endogenous truth drifted for sample `{}`", case.sample_id));
        }
    }

    let checked_samples =
        actual.sample_truths.iter().map(|sample| sample.sample_id.clone()).collect::<Vec<_>>();
    let validated_total_reads =
        actual.sample_truths.iter().map(|sample| sample.summary.total_reads).sum::<u64>();
    let validated_contaminant_reads =
        actual.sample_truths.iter().map(|sample| sample.summary.contaminant_reads).sum::<u64>();
    let validated_retained_reads =
        actual.sample_truths.iter().map(|sample| sample.summary.retained_reads).sum::<u64>();

    Ok(BamEndogenousTruthValidationReport {
        schema_version: BAM_ENDOGENOUS_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_sample_count: checked_samples.len(),
        validated_total_reads,
        validated_contaminant_reads,
        validated_retained_reads,
        checked_samples,
        valid: true,
    })
}
