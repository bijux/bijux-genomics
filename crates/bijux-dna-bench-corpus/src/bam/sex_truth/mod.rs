//! Compare complete sex-inference fixture records and required call-class coverage.
use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path as resolve_fixture_path};
use anyhow::{anyhow, Result};
use bijux_dna_domain_bam::metrics::SexConfidenceClass;
use std::collections::BTreeMap;
use std::path::Path;
mod admission;
mod class_coverage;
mod expected;
mod model;
mod observations;
use admission::validate_manifest_contract;
use class_coverage::{collect_call_classes, validate_required_call_classes};
use expected::validate_bundle_contract;
pub use model::BamSexTruthValidationReport;
use observations::{
    build_actual_truth_bundle, load_bam_sex_truth_bundle, load_bam_sex_truth_manifest_path,
};

pub const BAM_SEX_TRUTH_FIXTURE_ID: &str = "sex-inference-truth";
pub const BAM_SEX_TRUTH_MANIFEST_SCHEMA_VERSION: &str = "bijux.bench.bam_sex_truth.v1";
const BAM_SEX_TRUTH_BUNDLE_SCHEMA_VERSION: &str = "bijux.bench.bam_sex_truth.expected.v1";
const BAM_SEX_TRUTH_VALIDATION_SCHEMA_VERSION: &str = "bijux.bench.bam_sex_truth.validation.v1";

/// Validate the governed sex-inference fixture truth.
///
/// # Errors
/// Returns the existing declaration, expected-record or local observation error.
pub fn validate_bam_sex_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamSexTruthValidationReport> {
    let manifest = load_bam_sex_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("BAM sex truth manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("BAM sex truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_bam_sex_truth_bundle(&expected_path)?;
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
            "BAM sex truth case count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for case in &manifest.cases {
        let expected_sample = expected_map.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!("expected BAM sex truth is missing sample `{}`", case.sample_id)
        })?;
        let actual_sample = actual_map.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!("observed BAM sex truth is missing sample `{}`", case.sample_id)
        })?;
        if expected_sample != actual_sample {
            return Err(anyhow!(
                "BAM sex truth drifted for sample `{}`\nexpected: {expected_sample:#?}\nobserved: {actual_sample:#?}",
                case.sample_id
            ));
        }
    }

    let validated_call_classes = collect_call_classes(&actual.sample_truths);
    validate_required_call_classes(&validated_call_classes, manifest_path)?;
    let validated_ok_case_count =
        actual.sample_truths.iter().filter(|sample| sample.summary.status == "ok").count();
    let validated_insufficient_case_count = actual
        .sample_truths
        .iter()
        .filter(|sample| sample.summary.call == SexConfidenceClass::Insufficient)
        .count();

    Ok(BamSexTruthValidationReport {
        schema_version: BAM_SEX_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.sample_truths.len(),
        validated_ok_case_count,
        validated_insufficient_case_count,
        validated_call_classes,
        checked_samples: manifest.cases.iter().map(|case| case.sample_id.clone()).collect(),
        valid: true,
    })
}
