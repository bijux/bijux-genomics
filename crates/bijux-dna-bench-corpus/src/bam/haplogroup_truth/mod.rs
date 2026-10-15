//! Compare complete haplogroup fixture records and required status coverage.
use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path as resolve_fixture_path};
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::Path;
mod admission;
mod expected;
mod model;
mod observations;
mod status_coverage;
use admission::validate_manifest_contract;
use expected::validate_bundle_contract;
pub use model::BamHaplogroupTruthValidationReport;
use observations::{
    build_actual_truth_bundle, load_bam_haplogroup_truth_bundle,
    load_bam_haplogroup_truth_manifest_path,
};
use status_coverage::{collect_statuses, validate_required_statuses};

pub const BAM_HAPLOGROUP_TRUTH_FIXTURE_ID: &str = "haplogroup-truth";
pub const BAM_HAPLOGROUP_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.bam_haplogroup_truth.v1";
const BAM_HAPLOGROUP_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.bam_haplogroup_truth.expected.v1";
const BAM_HAPLOGROUP_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_haplogroup_truth.validation.v1";

/// Validate the governed haplogroup fixture truth.
///
/// # Errors
/// Returns the existing declaration, expected-record or local observation error.
pub fn validate_bam_haplogroup_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamHaplogroupTruthValidationReport> {
    let manifest = load_bam_haplogroup_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM haplogroup truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("BAM haplogroup truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_bam_haplogroup_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    let expected_map = expected
        .case_truths
        .iter()
        .map(|case| (case.case_id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    let actual_map = actual
        .case_truths
        .iter()
        .map(|case| (case.case_id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    if expected_map.len() != actual_map.len() {
        return Err(anyhow!(
            "BAM haplogroup truth case count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for case in &manifest.cases {
        let expected_case = expected_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("expected BAM haplogroup truth is missing case `{}`", case.case_id)
        })?;
        let actual_case = actual_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("observed BAM haplogroup truth is missing case `{}`", case.case_id)
        })?;
        if expected_case != actual_case {
            return Err(anyhow!(
                "BAM haplogroup truth drifted for case `{}`\nexpected: {expected_case:#?}\nobserved: {actual_case:#?}",
                case.case_id
            ));
        }
    }

    let validated_statuses = collect_statuses(&actual.case_truths);
    validate_required_statuses(&validated_statuses, manifest_path)?;

    Ok(BamHaplogroupTruthValidationReport {
        schema_version: BAM_HAPLOGROUP_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.case_truths.len(),
        validated_ready_case_count: actual
            .case_truths
            .iter()
            .filter(|case| case.summary.status == "ready")
            .count(),
        validated_uncertain_case_count: actual
            .case_truths
            .iter()
            .filter(|case| case.summary.status == "uncertain_marker_support")
            .count(),
        validated_coverage_gate_case_count: actual
            .case_truths
            .iter()
            .filter(|case| case.summary.status == "coverage_gate_not_met")
            .count(),
        validated_statuses,
        checked_cases: manifest.cases.iter().map(|case| case.case_id.clone()).collect(),
        valid: true,
    })
}
