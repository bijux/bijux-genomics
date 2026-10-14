//! Validate declared VCF filter fixture truth against canonical domain observations.
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::Path;

mod admission;
mod model;
mod observations;
use admission::{validate_bundle_contract, validate_manifest_contract};
pub use model::VcfFilterTruthValidationReport;
use model::{
    VcfDamageFilterMetricsTruth, VcfFilterMetricsTruth, VcfFilterTruthBundle, VcfFilterTruthCase,
    VcfFilterTruthCaseTruth, VcfFilterTruthKind, VcfFilterTruthManifest,
};
use observations::{
    build_actual_truth_bundle, load_vcf_filter_truth_bundle, load_vcf_filter_truth_manifest_path,
    path_relative_to_repo, resolve_fixture_path, resolve_repo_relative_path,
};

pub const VCF_FILTER_TRUTH_FIXTURE_ID: &str = "vcf-filter-truth";
pub const VCF_FILTER_TRUTH_MANIFEST_SCHEMA_VERSION: &str = "bijux.bench.vcf_filter_truth.v1";
const VCF_FILTER_TRUTH_BUNDLE_SCHEMA_VERSION: &str = "bijux.bench.vcf_filter_truth.expected.v1";
const VCF_FILTER_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.vcf_filter_truth.validation.v1";

/// Admit a declared filter-truth fixture without writing or running tools.
///
/// # Errors
/// Returns the existing source, fixture-support, or expected-versus-observed truth error.
pub fn validate_vcf_filter_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<VcfFilterTruthValidationReport> {
    let manifest = load_vcf_filter_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("VCF filter truth manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("VCF filter truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_vcf_filter_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    let expected_map =
        expected.cases.iter().map(|case| (case.case_id.as_str(), case)).collect::<BTreeMap<_, _>>();
    let actual_map =
        actual.cases.iter().map(|case| (case.case_id.as_str(), case)).collect::<BTreeMap<_, _>>();
    if expected_map.len() != actual_map.len() {
        return Err(anyhow!(
            "VCF filter truth case count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for case in &manifest.cases {
        let expected_case = expected_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("expected VCF filter truth is missing case `{}`", case.case_id)
        })?;
        let actual_case = actual_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("observed VCF filter truth is missing case `{}`", case.case_id)
        })?;
        if expected_case != actual_case {
            return Err(anyhow!(
                "VCF filter truth drifted for case `{}`\nexpected: {expected_case:#?}\nobserved: {actual_case:#?}",
                case.case_id
            ));
        }
    }

    Ok(VcfFilterTruthValidationReport {
        schema_version: VCF_FILTER_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.cases.len(),
        validated_stage_ids: collect_stage_ids(&actual.cases),
        validated_tool_ids: collect_tool_ids(&actual.cases),
        valid: true,
    })
}

fn collect_stage_ids(cases: &[VcfFilterTruthCaseTruth]) -> Vec<String> {
    let mut values = cases
        .iter()
        .map(|case| {
            let Some(stage_id) =
                case.filter_metrics.as_ref().map(|metrics| metrics.stage_id.clone()).or_else(
                    || case.damage_filter_metrics.as_ref().map(|metrics| metrics.stage_id.clone()),
                )
            else {
                panic!("vcf-filter truth case `{}` must carry metrics", case.case_id);
            };
            stage_id
        })
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn collect_tool_ids(cases: &[VcfFilterTruthCaseTruth]) -> Vec<String> {
    let mut values = cases
        .iter()
        .map(|case| {
            let Some(tool_id) =
                case.filter_metrics.as_ref().map(|metrics| metrics.tool_id.clone()).or_else(|| {
                    case.damage_filter_metrics.as_ref().map(|metrics| metrics.tool_id.clone())
                })
            else {
                panic!("vcf-filter truth case `{}` must carry metrics", case.case_id);
            };
            tool_id
        })
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}
