#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::{
    parse_angsd_stage_metrics, parse_bcftools_stage_metrics, summarize_vcf_filter_output_truth,
    VcfDomainStage, VcfFilterOutputTruthSummaryV1,
};
use serde::{Deserialize, Serialize};

pub(crate) const VCF_FILTER_TRUTH_FIXTURE_ID: &str = "vcf-filter-truth";
pub(crate) const VCF_FILTER_TRUTH_MANIFEST_SCHEMA_VERSION: &str = "bijux.bench.vcf_filter_truth.v1";
const VCF_FILTER_TRUTH_BUNDLE_SCHEMA_VERSION: &str = "bijux.bench.vcf_filter_truth.expected.v1";
const VCF_FILTER_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.vcf_filter_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct VcfFilterTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    expected_path: PathBuf,
    source_paths: Vec<PathBuf>,
    cases: Vec<VcfFilterTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct VcfFilterTruthCase {
    case_id: String,
    stage_id: String,
    tool_id: String,
    artifact_root_path: PathBuf,
    #[serde(default)]
    output_vcf_path: Option<PathBuf>,
    truth_kind: VcfFilterTruthKind,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum VcfFilterTruthKind {
    FilterLabels,
    DamageRemoval,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct VcfFilterTruthValidationReport {
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
struct VcfFilterTruthBundle {
    schema_version: String,
    fixture_id: String,
    cases: Vec<VcfFilterTruthCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfFilterTruthCaseTruth {
    case_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    filter_metrics: Option<VcfFilterMetricsTruth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    damage_filter_metrics: Option<VcfDamageFilterMetricsTruth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    output_truth: Option<VcfFilterOutputTruthSummaryV1>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfFilterMetricsTruth {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    input_variants: u64,
    pass_variants: u64,
    failed_variants: u64,
    filter_ids: Vec<String>,
    depth_threshold: f64,
    quality_threshold: f64,
    missingness_threshold: f64,
    sample_count: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfDamageFilterMetricsTruth {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    input_variants: u64,
    removed_variants: u64,
    retained_variants: u64,
    low_quality_filtered_variants: u64,
    damage_ratio_filtered_variants: u64,
    terminal_damage_filtered_variants: u64,
    damage_context_rule: String,
    terminal_context_count: u64,
    sample_count: u64,
}

pub(crate) fn validate_vcf_filter_truth_manifest_path(
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

fn load_vcf_filter_truth_manifest_path(manifest_path: &Path) -> Result<VcfFilterTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
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
        ("bcftools", VcfDomainStage::Filter, VcfFilterTruthKind::FilterLabels) => {}
        ("bcftools", VcfDomainStage::DamageFilter, VcfFilterTruthKind::DamageRemoval) => {}
        ("angsd", VcfDomainStage::DamageFilter, VcfFilterTruthKind::DamageRemoval) => {}
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

fn load_vcf_filter_truth_bundle(expected_path: &Path) -> Result<VcfFilterTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
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

fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &VcfFilterTruthManifest,
) -> Result<VcfFilterTruthBundle> {
    let mut cases = Vec::with_capacity(manifest.cases.len());
    for case in &manifest.cases {
        let stage = parse_supported_stage(&case.stage_id)?;
        let artifact_root = resolve_repo_relative_path(repo_root, &case.artifact_root_path);
        let output_truth = case
            .output_vcf_path
            .as_ref()
            .map(|path| -> Result<VcfFilterOutputTruthSummaryV1> {
                let resolved = resolve_repo_relative_path(repo_root, path);
                let mut summary =
                    summarize_vcf_filter_output_truth(&resolved, &case.stage_id, &case.tool_id)?;
                summary.input_vcf = PathBuf::from(path_relative_to_repo(repo_root, &resolved));
                Ok(summary)
            })
            .transpose()?;
        let case_truth = match case.truth_kind {
            VcfFilterTruthKind::FilterLabels => VcfFilterTruthCaseTruth {
                case_id: case.case_id.clone(),
                filter_metrics: Some(parse_filter_metrics_truth(
                    &case.tool_id,
                    stage,
                    &artifact_root,
                )?),
                damage_filter_metrics: None,
                output_truth,
            },
            VcfFilterTruthKind::DamageRemoval => VcfFilterTruthCaseTruth {
                case_id: case.case_id.clone(),
                filter_metrics: None,
                damage_filter_metrics: Some(parse_damage_filter_metrics_truth(
                    &case.tool_id,
                    stage,
                    &artifact_root,
                )?),
                output_truth,
            },
        };
        cases.push(case_truth);
    }
    Ok(VcfFilterTruthBundle {
        schema_version: VCF_FILTER_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        cases,
    })
}

fn parse_filter_metrics_truth(
    tool_id: &str,
    stage: VcfDomainStage,
    artifact_root: &Path,
) -> Result<VcfFilterMetricsTruth> {
    let raw = match tool_id {
        "bcftools" => parse_bcftools_stage_metrics(stage, artifact_root)?,
        other => {
            return Err(anyhow!(
                "VCF filter truth does not support filter metrics for tool `{other}`"
            ))
        }
    };
    serde_json::from_value(raw).context("deserialize VCF filter metrics truth")
}

fn parse_damage_filter_metrics_truth(
    tool_id: &str,
    stage: VcfDomainStage,
    artifact_root: &Path,
) -> Result<VcfDamageFilterMetricsTruth> {
    let raw = match tool_id {
        "bcftools" => parse_bcftools_stage_metrics(stage, artifact_root)?,
        "angsd" => parse_angsd_stage_metrics(stage, artifact_root)?,
        other => {
            return Err(anyhow!(
                "VCF filter truth does not support damage-filter metrics for tool `{other}`"
            ))
        }
    };
    serde_json::from_value(raw).context("deserialize VCF damage-filter metrics truth")
}

fn parse_supported_stage(stage_id: &str) -> Result<VcfDomainStage> {
    match stage_id {
        "vcf.filter" => Ok(VcfDomainStage::Filter),
        "vcf.damage_filter" => Ok(VcfDomainStage::DamageFilter),
        other => Err(anyhow!("unsupported VCF filter truth stage `{other}`")),
    }
}

fn collect_stage_ids(cases: &[VcfFilterTruthCaseTruth]) -> Vec<String> {
    let mut values = cases
        .iter()
        .map(|case| {
            case.filter_metrics
                .as_ref()
                .map(|metrics| metrics.stage_id.clone())
                .or_else(|| {
                    case.damage_filter_metrics.as_ref().map(|metrics| metrics.stage_id.clone())
                })
                .expect("case truth must carry metrics")
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
            case.filter_metrics
                .as_ref()
                .map(|metrics| metrics.tool_id.clone())
                .or_else(|| {
                    case.damage_filter_metrics.as_ref().map(|metrics| metrics.tool_id.clone())
                })
                .expect("case truth must carry metrics")
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
