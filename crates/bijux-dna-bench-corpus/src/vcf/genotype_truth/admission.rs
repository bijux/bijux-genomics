//! Declaration and expected-bundle admission precede native observations.
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};

use super::model::{
    VcfGenotypeEvaluationKind, VcfGenotypeTruthBundle, VcfGenotypeTruthCase,
    VcfGenotypeTruthManifest,
};
use super::{
    resolve_repo_relative_path, VCF_GENOTYPE_TRUTH_BUNDLE_SCHEMA_VERSION,
    VCF_GENOTYPE_TRUTH_FIXTURE_ID, VCF_GENOTYPE_TRUTH_MANIFEST_SCHEMA_VERSION,
};

pub(super) fn load_vcf_genotype_truth_manifest_path(
    manifest_path: &Path,
) -> Result<VcfGenotypeTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &VcfGenotypeTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != VCF_GENOTYPE_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "VCF genotype truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            VCF_GENOTYPE_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != VCF_GENOTYPE_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "VCF genotype truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            VCF_GENOTYPE_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "VCF genotype truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "VCF genotype truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF genotype truth source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "VCF genotype truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }

    let mut case_ids = BTreeSet::new();
    for case in &manifest.cases {
        validate_case_contract(repo_root, case, manifest_path)?;
        if !case_ids.insert(case.case_id.clone()) {
            return Err(anyhow!("VCF genotype truth manifest repeats case_id `{}`", case.case_id));
        }
    }
    Ok(())
}

fn validate_case_contract(
    repo_root: &Path,
    case: &VcfGenotypeTruthCase,
    manifest_path: &Path,
) -> Result<()> {
    if case.case_id.trim().is_empty() {
        return Err(anyhow!(
            "VCF genotype truth manifest `{}` contains an empty case_id",
            manifest_path.display()
        ));
    }
    if case.stage_id.trim().is_empty() || case.tool_id.trim().is_empty() {
        return Err(anyhow!(
            "VCF genotype truth case `{}` must declare non-empty stage_id and tool_id",
            case.case_id
        ));
    }
    let input_vcf_path = resolve_repo_relative_path(repo_root, &case.input_vcf_path);
    if !input_vcf_path.is_file() {
        return Err(anyhow!(
            "VCF genotype truth case `{}` input VCF is missing: {}",
            case.case_id,
            input_vcf_path.display()
        ));
    }
    if let Some(command_path) = &case.command_path {
        let resolved = resolve_repo_relative_path(repo_root, command_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF genotype truth case `{}` command file is missing: {}",
                case.case_id,
                resolved.display()
            ));
        }
    }
    match case.evaluation_kind {
        VcfGenotypeEvaluationKind::None | VcfGenotypeEvaluationKind::Likelihood => {}
        VcfGenotypeEvaluationKind::Diploid => {
            if case.declared_ploidy.as_deref() != Some("diploid") {
                return Err(anyhow!(
                    "VCF genotype truth diploid case `{}` must declare `diploid` ploidy",
                    case.case_id
                ));
            }
            if case.mean_coverage.is_none() || case.minimum_mean_coverage.is_none() {
                return Err(anyhow!(
                    "VCF genotype truth diploid case `{}` must declare mean coverage and minimum_mean_coverage",
                    case.case_id
                ));
            }
        }
        VcfGenotypeEvaluationKind::Pseudohaploid => {
            if !matches!(case.declared_ploidy.as_deref(), Some("haploid" | "pseudohaploid")) {
                return Err(anyhow!(
                    "VCF genotype truth pseudohaploid case `{}` must declare haploid-compatible ploidy",
                    case.case_id
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn load_vcf_genotype_truth_bundle(
    expected_path: &Path,
) -> Result<VcfGenotypeTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

pub(super) fn validate_bundle_contract(
    manifest: &VcfGenotypeTruthManifest,
    bundle: &VcfGenotypeTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != VCF_GENOTYPE_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "VCF genotype truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            VCF_GENOTYPE_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "VCF genotype truth bundle `{}` fixture_id `{}` must equal `{}`",
            expected_path.display(),
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let expected_case_ids =
        manifest.cases.iter().map(|case| case.case_id.as_str()).collect::<BTreeSet<_>>();
    let bundle_case_ids =
        bundle.cases.iter().map(|case| case.case_id.as_str()).collect::<BTreeSet<_>>();
    if expected_case_ids != bundle_case_ids {
        return Err(anyhow!(
            "VCF genotype truth bundle case ids do not match manifest `{}`",
            expected_path.display()
        ));
    }
    Ok(())
}
