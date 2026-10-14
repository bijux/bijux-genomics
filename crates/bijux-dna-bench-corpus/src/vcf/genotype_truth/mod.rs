//! Read-only admission and complete comparison of declared VCF genotype fixture truth.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

mod model;
use model::VcfGenotypeTruthCaseTruth;
pub use model::VcfGenotypeTruthValidationReport;
mod admission;
use admission::{
    load_vcf_genotype_truth_bundle, load_vcf_genotype_truth_manifest_path,
    validate_bundle_contract, validate_manifest_contract,
};
mod observations;
use observations::build_actual_truth_bundle;

pub const VCF_GENOTYPE_TRUTH_FIXTURE_ID: &str = "vcf-genotype-truth";
pub const VCF_GENOTYPE_TRUTH_MANIFEST_SCHEMA_VERSION: &str = "bijux.bench.vcf_genotype_truth.v1";
const VCF_GENOTYPE_TRUTH_BUNDLE_SCHEMA_VERSION: &str = "bijux.bench.vcf_genotype_truth.expected.v1";
const VCF_GENOTYPE_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.vcf_genotype_truth.validation.v1";

/// Compare declared genotype fixture expectations with their local observations.
///
/// # Errors
/// Returns an error when declared files cannot be read or decoded, admission fails,
/// or complete observed case truth differs from the expected bundle.
pub fn validate_vcf_genotype_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<VcfGenotypeTruthValidationReport> {
    let manifest = load_vcf_genotype_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("VCF genotype truth manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("VCF genotype truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_vcf_genotype_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    let expected_map =
        expected.cases.iter().map(|case| (case.case_id.as_str(), case)).collect::<BTreeMap<_, _>>();
    let actual_map =
        actual.cases.iter().map(|case| (case.case_id.as_str(), case)).collect::<BTreeMap<_, _>>();
    if expected_map.len() != actual_map.len() {
        return Err(anyhow!(
            "VCF genotype truth case count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for case in &manifest.cases {
        let expected_case = expected_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("expected VCF genotype truth is missing case `{}`", case.case_id)
        })?;
        let actual_case = actual_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("observed VCF genotype truth is missing case `{}`", case.case_id)
        })?;
        if expected_case != actual_case {
            return Err(anyhow!(
                "VCF genotype truth drifted for case `{}`\nexpected: {expected_case:#?}\nobserved: {actual_case:#?}",
                case.case_id
            ));
        }
    }

    let validated_stage_ids = collect_stage_ids(&actual.cases);
    let validated_tool_ids = collect_tool_ids(&actual.cases);
    Ok(VcfGenotypeTruthValidationReport {
        schema_version: VCF_GENOTYPE_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.cases.len(),
        validated_stage_ids,
        validated_tool_ids,
        valid: true,
    })
}

fn collect_stage_ids(cases: &[VcfGenotypeTruthCaseTruth]) -> Vec<String> {
    let mut values = cases.iter().map(|case| case.summary.stage_id.clone()).collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn collect_tool_ids(cases: &[VcfGenotypeTruthCaseTruth]) -> Vec<String> {
    let mut values = cases.iter().map(|case| case.summary.tool_id.clone()).collect::<Vec<_>>();
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
