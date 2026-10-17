//! Read-only admission of declared population fixture expectations and native observations.
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
mod admission;
mod model;
mod observations;
use admission::{load_bundle, load_manifest, validate_bundle_contract, validate_manifest_contract};
use model::PopulationStructureTruthCaseTruth;
pub use model::PopulationStructureTruthValidationReport;
use observations::build_actual_truth_bundle;

pub const POPULATION_STRUCTURE_TRUTH_FIXTURE_ID: &str = "population-structure-truth";
pub const POPULATION_STRUCTURE_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.population_structure_truth.v1";
const POPULATION_STRUCTURE_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.population_structure_truth.expected.v1";
const POPULATION_STRUCTURE_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.population_structure_truth.validation.v1";

/// Compare declared population expectations with local native metrics and metadata joins.
///
/// # Errors
/// Returns an error when declarations or evidence cannot be read or decoded,
/// admission fails, or complete observed case truth differs from the expected bundle.
pub fn validate_population_structure_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<PopulationStructureTruthValidationReport> {
    let manifest = load_manifest(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "population-structure truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!(
            "population-structure truth bundle is missing: {}",
            expected_path.display()
        ));
    }

    let expected = load_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    let expected_map =
        expected.cases.iter().map(|case| (case.case_id.as_str(), case)).collect::<BTreeMap<_, _>>();
    let actual_map =
        actual.cases.iter().map(|case| (case.case_id.as_str(), case)).collect::<BTreeMap<_, _>>();
    if expected_map.len() != actual_map.len() {
        return Err(anyhow!(
            "population-structure truth case count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for case in &manifest.cases {
        let expected_case = expected_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("expected population-structure truth is missing case `{}`", case.case_id)
        })?;
        let actual_case = actual_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("observed population-structure truth is missing case `{}`", case.case_id)
        })?;
        if expected_case != actual_case {
            return Err(anyhow!(
                "population-structure truth drifted for case `{}`\nexpected: {expected_case:#?}\nobserved: {actual_case:#?}",
                case.case_id
            ));
        }
    }

    Ok(PopulationStructureTruthValidationReport {
        schema_version: POPULATION_STRUCTURE_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.cases.len(),
        validated_stage_ids: collect_stage_ids(&actual.cases),
        validated_tool_ids: collect_tool_ids(&actual.cases),
        valid: true,
    })
}

fn collect_stage_ids(cases: &[PopulationStructureTruthCaseTruth]) -> Vec<String> {
    let mut values = cases
        .iter()
        .map(|case| {
            let Some(stage_id) = case
                .pca_metrics
                .as_ref()
                .and_then(|metrics| metrics.get("stage_id").and_then(serde_json::Value::as_str))
                .or_else(|| {
                    case.admixture_metrics.as_ref().and_then(|metrics| {
                        metrics.get("stage_id").and_then(serde_json::Value::as_str)
                    })
                })
                .or_else(|| {
                    case.population_structure_metrics.as_ref().and_then(|metrics| {
                        metrics.get("stage_id").and_then(serde_json::Value::as_str)
                    })
                })
            else {
                panic!("population-structure truth case `{}` must carry stage id", case.case_id);
            };
            stage_id.to_string()
        })
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn collect_tool_ids(cases: &[PopulationStructureTruthCaseTruth]) -> Vec<String> {
    let mut values = cases
        .iter()
        .map(|case| {
            let Some(tool_id) = case
                .pca_metrics
                .as_ref()
                .and_then(|metrics| metrics.get("tool_id").and_then(serde_json::Value::as_str))
                .or_else(|| {
                    case.admixture_metrics.as_ref().and_then(|metrics| {
                        metrics.get("tool_id").and_then(serde_json::Value::as_str)
                    })
                })
                .or_else(|| {
                    case.population_structure_metrics.as_ref().and_then(|metrics| {
                        metrics.get("tool_id").and_then(serde_json::Value::as_str)
                    })
                })
            else {
                panic!("population-structure truth case `{}` must carry tool id", case.case_id);
            };
            tool_id.to_string()
        })
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn resolve_repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn resolve_fixture_path(fixture_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        fixture_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
