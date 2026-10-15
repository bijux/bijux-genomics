//! Validate declared aDNA damage fixture truth without execution or publication.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{anyhow, Result};

use crate::paths::{path_relative_to_repo, resolve_manifest_relative_path as resolve_fixture_path};

mod admission;
mod model;
mod observations;

use admission::{validate_bundle_contract, validate_manifest_contract};
pub use model::AdnaDamageTruthValidationReport;
use observations::{
    build_actual_truth_bundle, load_adna_damage_truth_bundle, load_adna_damage_truth_manifest_path,
};

pub const ADNA_DAMAGE_TRUTH_FIXTURE_ID: &str = "adna-damage-truth";
pub const ADNA_DAMAGE_TRUTH_MANIFEST_SCHEMA_VERSION: &str = "bijux.bench.adna_damage_truth.v1";
const ADNA_DAMAGE_TRUTH_BUNDLE_SCHEMA_VERSION: &str = "bijux.bench.adna_damage_truth.expected.v1";
const ADNA_DAMAGE_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.adna_damage_truth.validation.v1";

/// Validate a damage fixture declaration against its expected and observed records.
///
/// # Errors
/// Returns the existing declaration, expected bundle or canonical domain observation error.
pub fn validate_adna_damage_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<AdnaDamageTruthValidationReport> {
    let manifest = load_adna_damage_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("aDNA damage truth manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("aDNA damage truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_adna_damage_truth_bundle(&expected_path)?;
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
            "aDNA damage truth case count drifted: expected {}, observed {}",
            expected_truths.len(),
            actual_truths.len()
        ));
    }
    for case in &manifest.cases {
        let expected_truth = expected_truths.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("expected aDNA damage truth is missing case `{}`", case.case_id)
        })?;
        let actual_truth = actual_truths.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("observed aDNA damage truth is missing case `{}`", case.case_id)
        })?;
        if expected_truth != actual_truth {
            return Err(anyhow!("aDNA damage truth drifted for case `{}`", case.case_id));
        }
    }

    let terminal_pattern_classes = actual
        .truths
        .iter()
        .map(|truth| truth.summary.terminal_pattern_class.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let validated_insufficient_case_count =
        actual.truths.iter().filter(|truth| truth.summary.insufficiency_reason.is_some()).count();

    Ok(AdnaDamageTruthValidationReport {
        schema_version: ADNA_DAMAGE_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.truths.len(),
        validated_insufficient_case_count,
        terminal_pattern_classes,
        checked_cases: manifest.cases.iter().map(|case| case.case_id.clone()).collect(),
        valid: true,
    })
}
