#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::{
    parse_eigensoft_stage_metrics, parse_plink2_stage_metrics,
    summarize_vcf_admixture_output_truth, summarize_vcf_pca_output_truth,
    summarize_vcf_population_structure_output_truth, VcfAdmixtureOutputTruthSummaryV1,
    VcfDomainStage, VcfPcaOutputTruthSummaryV1, VcfPopulationStructureOutputTruthSummaryV1,
};
use serde::{Deserialize, Serialize};

pub(crate) const POPULATION_STRUCTURE_TRUTH_FIXTURE_ID: &str = "population-structure-truth";
pub(crate) const POPULATION_STRUCTURE_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.population_structure_truth.v1";
const POPULATION_STRUCTURE_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.population_structure_truth.expected.v1";
const POPULATION_STRUCTURE_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.population_structure_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PopulationStructureTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    expected_path: PathBuf,
    source_paths: Vec<PathBuf>,
    cases: Vec<PopulationStructureTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PopulationStructureTruthCase {
    case_id: String,
    stage_id: String,
    tool_id: String,
    artifact_root_path: PathBuf,
    sample_metadata_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PopulationStructureTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_case_count: usize,
    pub(crate) validated_stage_ids: Vec<String>,
    pub(crate) validated_tool_ids: Vec<String>,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct PopulationStructureTruthBundle {
    schema_version: String,
    fixture_id: String,
    cases: Vec<PopulationStructureTruthCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct PopulationStructureTruthCaseTruth {
    case_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pca_metrics: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    admixture_metrics: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    population_structure_metrics: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pca_output_truth: Option<VcfPcaOutputTruthSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    admixture_output_truth: Option<VcfAdmixtureOutputTruthSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    population_structure_output_truth: Option<VcfPopulationStructureOutputTruthSummaryV1>,
}

pub(crate) fn validate_population_structure_truth_manifest_path(
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

fn load_manifest(manifest_path: &Path) -> Result<PopulationStructureTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
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
        ) => Ok(()),
        ("eigensoft", VcfDomainStage::Pca | VcfDomainStage::PopulationStructure) => Ok(()),
        _ => Err(anyhow!(
            "population-structure truth case `{}` does not support tool `{}` on stage `{}`",
            case.case_id,
            case.tool_id,
            case.stage_id
        )),
    }
}

fn parse_supported_stage(stage_id: &str) -> Result<VcfDomainStage> {
    match stage_id {
        "vcf.pca" => Ok(VcfDomainStage::Pca),
        "vcf.admixture" => Ok(VcfDomainStage::Admixture),
        "vcf.population_structure" => Ok(VcfDomainStage::PopulationStructure),
        _ => Err(anyhow!(
            "population-structure truth only supports `vcf.pca`, `vcf.admixture`, and `vcf.population_structure`; found `{stage_id}`"
        )),
    }
}

fn load_bundle(expected_path: &Path) -> Result<PopulationStructureTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
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

fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &PopulationStructureTruthManifest,
) -> Result<PopulationStructureTruthBundle> {
    let cases = manifest
        .cases
        .iter()
        .map(|case| {
            let stage = parse_supported_stage(&case.stage_id)?;
            let artifact_root = resolve_repo_relative_path(repo_root, &case.artifact_root_path);
            let metadata_path = resolve_repo_relative_path(repo_root, &case.sample_metadata_path);
            let metrics = match case.tool_id.as_str() {
                "plink2" => parse_plink2_stage_metrics(stage, &artifact_root)?,
                "eigensoft" => parse_eigensoft_stage_metrics(stage, &artifact_root)?,
                other => {
                    return Err(anyhow!(
                        "population-structure truth case `{}` does not support tool `{other}`",
                        case.case_id
                    ))
                }
            };
            let case_truth = match stage {
                VcfDomainStage::Pca => {
                    let mut output_truth =
                        summarize_vcf_pca_output_truth(&metrics, &metadata_path)?;
                    output_truth.sample_metadata_path =
                        PathBuf::from(path_relative_to_repo(repo_root, &metadata_path));
                    PopulationStructureTruthCaseTruth {
                        case_id: case.case_id.clone(),
                        pca_metrics: Some(metrics),
                        admixture_metrics: None,
                        population_structure_metrics: None,
                        pca_output_truth: Some(output_truth),
                        admixture_output_truth: None,
                        population_structure_output_truth: None,
                    }
                }
                VcfDomainStage::Admixture => {
                    let mut output_truth =
                        summarize_vcf_admixture_output_truth(&metrics, &metadata_path)?;
                    output_truth.sample_metadata_path =
                        PathBuf::from(path_relative_to_repo(repo_root, &metadata_path));
                    PopulationStructureTruthCaseTruth {
                        case_id: case.case_id.clone(),
                        pca_metrics: None,
                        admixture_metrics: Some(metrics),
                        population_structure_metrics: None,
                        pca_output_truth: None,
                        admixture_output_truth: Some(output_truth),
                        population_structure_output_truth: None,
                    }
                }
                VcfDomainStage::PopulationStructure => {
                    let mut output_truth =
                        summarize_vcf_population_structure_output_truth(&metrics, &metadata_path)?;
                    output_truth.sample_metadata_path =
                        PathBuf::from(path_relative_to_repo(repo_root, &metadata_path));
                    PopulationStructureTruthCaseTruth {
                        case_id: case.case_id.clone(),
                        pca_metrics: None,
                        admixture_metrics: None,
                        population_structure_metrics: Some(metrics),
                        pca_output_truth: None,
                        admixture_output_truth: None,
                        population_structure_output_truth: Some(output_truth),
                    }
                }
                _ => unreachable!("stage already constrained"),
            };
            Ok(case_truth)
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(PopulationStructureTruthBundle {
        schema_version: POPULATION_STRUCTURE_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        cases,
    })
}

fn collect_stage_ids(cases: &[PopulationStructureTruthCaseTruth]) -> Vec<String> {
    let mut values = cases
        .iter()
        .map(|case| {
            case.pca_metrics
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
                .expect("case truth must carry stage id")
                .to_string()
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
            case.pca_metrics
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
                .expect("case truth must carry tool id")
                .to_string()
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
