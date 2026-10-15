//! Compare complete declared coverage and GC-bias fixture records.
use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path as resolve_fixture_path};
use anyhow::{anyhow, Result};
use std::path::Path;
mod admission;
mod comparison;
mod expected;
mod model;
mod observations;
use admission::validate_manifest_contract;
use comparison::{validate_coverage_sample_truths, validate_gc_bias_sample_truths};
use expected::validate_bundle_contract;
pub use model::BamGcCoverageTruthValidationReport;
use observations::{
    build_actual_truth_bundle, load_bam_gc_coverage_truth_bundle,
    load_bam_gc_coverage_truth_manifest_path,
};

pub const BAM_GC_COVERAGE_TRUTH_FIXTURE_ID: &str = "bam-gc-coverage-truth";
pub const BAM_GC_COVERAGE_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.bam_gc_coverage_truth.v1";
const BAM_GC_COVERAGE_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.bam_gc_coverage_truth.expected.v1";
const BAM_GC_COVERAGE_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_gc_coverage_truth.validation.v1";

/// Validate the governed coverage and GC-bias fixture truth.
///
/// # Errors
/// Returns the existing declaration, expected-record or local observation error.
pub fn validate_bam_gc_coverage_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamGcCoverageTruthValidationReport> {
    let manifest = load_bam_gc_coverage_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM GC-bias/coverage truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle is missing: {}",
            expected_path.display()
        ));
    }

    let expected = load_bam_gc_coverage_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    validate_coverage_sample_truths(&expected.coverage_truths, &actual.coverage_truths)?;
    validate_gc_bias_sample_truths(&expected.gc_bias_truths, &actual.gc_bias_truths)?;

    let checked_samples = manifest
        .coverage_cases
        .iter()
        .map(|case| case.sample_id.clone())
        .chain(manifest.gc_bias_cases.iter().map(|case| case.sample_id.clone()))
        .collect::<Vec<_>>();
    let validated_coverage_region_count =
        actual.coverage_truths.iter().map(|sample| sample.summary.region_summaries.len()).sum();
    let validated_gc_bias_bin_count =
        actual.gc_bias_truths.iter().map(|sample| sample.summary.gc_bins.len()).sum();

    Ok(BamGcCoverageTruthValidationReport {
        schema_version: BAM_GC_COVERAGE_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_coverage_sample_count: actual.coverage_truths.len(),
        validated_gc_bias_sample_count: actual.gc_bias_truths.len(),
        validated_coverage_region_count,
        validated_gc_bias_bin_count,
        valid: true,
        checked_samples,
    })
}
