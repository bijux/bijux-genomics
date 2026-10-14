//! Local reference and supported-case admission precede native observations.
use super::model::{
    PopulationStructureTruthBundle, PopulationStructureTruthCase, PopulationStructureTruthManifest,
};
use super::{
    resolve_repo_relative_path, POPULATION_STRUCTURE_TRUTH_BUNDLE_SCHEMA_VERSION,
    POPULATION_STRUCTURE_TRUTH_FIXTURE_ID, POPULATION_STRUCTURE_TRUTH_MANIFEST_SCHEMA_VERSION,
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::VcfDomainStage;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

pub(super) fn load_manifest(manifest_path: &Path) -> Result<PopulationStructureTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &PopulationStructureTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != POPULATION_STRUCTURE_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "population-structure truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            POPULATION_STRUCTURE_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != POPULATION_STRUCTURE_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "population-structure truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            POPULATION_STRUCTURE_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "population-structure truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "population-structure truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.exists() {
            return Err(anyhow!(
                "population-structure truth source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "population-structure truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }
    let mut case_ids = BTreeSet::new();
    for case in &manifest.cases {
        validate_case_contract(repo_root, case, manifest_path)?;
        if !case_ids.insert(case.case_id.clone()) {
            return Err(anyhow!(
                "population-structure truth manifest repeats case_id `{}`",
                case.case_id
            ));
        }
    }
    Ok(())
}

fn validate_case_contract(
    repo_root: &Path,
    case: &PopulationStructureTruthCase,
    manifest_path: &Path,
) -> Result<()> {
    if case.case_id.trim().is_empty()
        || case.stage_id.trim().is_empty()
        || case.tool_id.trim().is_empty()
    {
        return Err(anyhow!(
            "population-structure truth manifest `{}` contains a case with empty identity fields",
            manifest_path.display()
        ));
    }
    let artifact_root = resolve_repo_relative_path(repo_root, &case.artifact_root_path);
    if !artifact_root.is_dir() {
        return Err(anyhow!(
            "population-structure truth case `{}` artifact_root_path is missing: {}",
            case.case_id,
            artifact_root.display()
        ));
    }
    let metadata_path = resolve_repo_relative_path(repo_root, &case.sample_metadata_path);
    if !metadata_path.is_file() {
        return Err(anyhow!(
            "population-structure truth case `{}` sample_metadata_path is missing: {}",
            case.case_id,
            metadata_path.display()
        ));
    }
    let stage = parse_supported_stage(&case.stage_id)?;
    match (case.tool_id.as_str(), stage) {
        (
            "plink2",
            VcfDomainStage::Pca | VcfDomainStage::Admixture | VcfDomainStage::PopulationStructure,
        )
        | ("eigensoft", VcfDomainStage::Pca | VcfDomainStage::PopulationStructure) => Ok(()),
        _ => Err(anyhow!(
            "population-structure truth case `{}` does not support tool `{}` on stage `{}`",
            case.case_id,
            case.tool_id,
            case.stage_id
        )),
    }
}

pub(super) fn parse_supported_stage(stage_id: &str) -> Result<VcfDomainStage> {
    match stage_id {
        "vcf.pca" => Ok(VcfDomainStage::Pca),
        "vcf.admixture" => Ok(VcfDomainStage::Admixture),
        "vcf.population_structure" => Ok(VcfDomainStage::PopulationStructure),
        _ => Err(anyhow!(
            "population-structure truth only supports `vcf.pca`, `vcf.admixture`, and `vcf.population_structure`; found `{stage_id}`"
        )),
    }
}

pub(super) fn load_bundle(expected_path: &Path) -> Result<PopulationStructureTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

pub(super) fn validate_bundle_contract(
    manifest: &PopulationStructureTruthManifest,
    bundle: &PopulationStructureTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != POPULATION_STRUCTURE_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "population-structure truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            POPULATION_STRUCTURE_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "population-structure truth bundle fixture_id `{}` must equal manifest fixture_id `{}`",
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
            "population-structure truth bundle case ids do not match manifest `{}`",
            expected_path.display()
        ));
    }
    Ok(())
}
