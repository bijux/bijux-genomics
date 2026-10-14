//! Admit declared sources, fixture case support and expected-bundle identities.
use super::{
    resolve_repo_relative_path, VcfFilterTruthBundle, VcfFilterTruthCase, VcfFilterTruthKind,
    VcfFilterTruthManifest, VCF_FILTER_TRUTH_BUNDLE_SCHEMA_VERSION, VCF_FILTER_TRUTH_FIXTURE_ID,
    VCF_FILTER_TRUTH_MANIFEST_SCHEMA_VERSION,
};
use anyhow::{anyhow, Result};
use bijux_dna_domain_vcf::VcfDomainStage;
use std::collections::BTreeSet;
use std::path::Path;

pub(super) fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &VcfFilterTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != VCF_FILTER_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "VCF filter truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            VCF_FILTER_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != VCF_FILTER_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "VCF filter truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            VCF_FILTER_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "VCF filter truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "VCF filter truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!("VCF filter truth source path is missing: {}", resolved.display()));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "VCF filter truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }

    let mut case_ids = BTreeSet::new();
    for case in &manifest.cases {
        validate_case_contract(repo_root, case, manifest_path)?;
        if !case_ids.insert(case.case_id.clone()) {
            return Err(anyhow!("VCF filter truth manifest repeats case_id `{}`", case.case_id));
        }
    }
    Ok(())
}

fn validate_case_contract(
    repo_root: &Path,
    case: &VcfFilterTruthCase,
    manifest_path: &Path,
) -> Result<()> {
    if case.case_id.trim().is_empty()
        || case.stage_id.trim().is_empty()
        || case.tool_id.trim().is_empty()
    {
        return Err(anyhow!(
            "VCF filter truth manifest `{}` contains a case with empty identity fields",
            manifest_path.display()
        ));
    }
    let artifact_root = resolve_repo_relative_path(repo_root, &case.artifact_root_path);
    if !artifact_root.is_dir() {
        return Err(anyhow!(
            "VCF filter truth case `{}` artifact root is missing: {}",
            case.case_id,
            artifact_root.display()
        ));
    }
    if let Some(output_vcf_path) = &case.output_vcf_path {
        let resolved = resolve_repo_relative_path(repo_root, output_vcf_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "VCF filter truth case `{}` output VCF is missing: {}",
                case.case_id,
                resolved.display()
            ));
        }
    }
    let stage = parse_supported_stage(&case.stage_id)?;
    match (case.tool_id.as_str(), stage, case.truth_kind) {
        ("bcftools", VcfDomainStage::Filter, VcfFilterTruthKind::FilterLabels)
        | ("bcftools" | "angsd", VcfDomainStage::DamageFilter, VcfFilterTruthKind::DamageRemoval) =>
            {}
        _ => {
            return Err(anyhow!(
                "VCF filter truth case `{}` uses unsupported tool/stage/truth combination `{}` / `{}` / {:?}",
                case.case_id,
                case.tool_id,
                case.stage_id,
                case.truth_kind
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_bundle_contract(
    manifest: &VcfFilterTruthManifest,
    bundle: &VcfFilterTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != VCF_FILTER_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "VCF filter truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            VCF_FILTER_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "VCF filter truth bundle `{}` fixture_id `{}` must equal `{}`",
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
            "VCF filter truth bundle case ids do not match manifest `{}`",
            expected_path.display()
        ));
    }
    Ok(())
}

pub(super) fn parse_supported_stage(stage_id: &str) -> Result<VcfDomainStage> {
    match stage_id {
        "vcf.filter" => Ok(VcfDomainStage::Filter),
        "vcf.damage_filter" => Ok(VcfDomainStage::DamageFilter),
        other => Err(anyhow!("unsupported VCF filter truth stage `{other}`")),
    }
}
