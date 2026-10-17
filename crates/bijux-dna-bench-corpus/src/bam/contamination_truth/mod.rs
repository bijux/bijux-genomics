//! Compare complete declared contamination fixture truth without execution or publication.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path as resolve_fixture_path};

mod admission;
mod model;
mod observations;

use admission::{validate_bundle_contract, validate_manifest_contract};
pub use model::AdnaContaminationTruthValidationReport;
use observations::{
    build_actual_truth_bundle, load_adna_contamination_truth_bundle,
    load_adna_contamination_truth_manifest_path,
};

pub const ADNA_CONTAMINATION_TRUTH_FIXTURE_ID: &str = "adna-contamination-truth";
pub const ADNA_CONTAMINATION_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.adna_contamination_truth.v1";
const ADNA_CONTAMINATION_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.adna_contamination_truth.expected.v1";
const ADNA_CONTAMINATION_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.adna_contamination_truth.validation.v1";

/// Validate declared contamination cases against expected and observed fixture records.
///
/// # Errors
/// Returns the existing declaration, expected bundle or domain observation error.
pub fn validate_adna_contamination_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<AdnaContaminationTruthValidationReport> {
    let manifest = load_adna_contamination_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "aDNA contamination truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!(
            "aDNA contamination truth bundle is missing: {}",
            expected_path.display()
        ));
    }

    let expected = load_adna_contamination_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    let expected_truths = expected
        .truths
        .iter()
        .map(|truth| (truth.case_id.as_str(), truth))
        .collect::<BTreeMap<_, _>>();
    let actual_truths = actual
        .truths
        .iter()
        .map(|truth| (truth.case_id.as_str(), truth))
        .collect::<BTreeMap<_, _>>();
    if expected_truths.len() != actual_truths.len() {
        return Err(anyhow!(
            "aDNA contamination truth case count drifted: expected {}, observed {}",
            expected_truths.len(),
            actual_truths.len()
        ));
    }
    for case in &manifest.cases {
        let expected_truth = expected_truths.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("expected aDNA contamination truth is missing case `{}`", case.case_id)
        })?;
        let actual_truth = actual_truths.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("observed aDNA contamination truth is missing case `{}`", case.case_id)
        })?;
        if expected_truth != actual_truth {
            return Err(anyhow!("aDNA contamination truth drifted for case `{}`", case.case_id));
        }
    }

    let tool_ids = actual
        .truths
        .iter()
        .map(|truth| truth.tool.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let validated_insufficient_case_count =
        actual.truths.iter().filter(|truth| truth.summary.status == "insufficient").count();

    Ok(AdnaContaminationTruthValidationReport {
        schema_version: ADNA_CONTAMINATION_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.truths.len(),
        validated_insufficient_case_count,
        tool_ids,
        checked_cases: manifest.cases.iter().map(|case| case.case_id.clone()).collect(),
        valid: true,
    })
}
