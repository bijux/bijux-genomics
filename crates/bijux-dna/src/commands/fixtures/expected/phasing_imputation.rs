#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::{
    parse_imputation_stage_metrics, parse_phasing_stage_metrics, summarize_vcf_imputation_output_truth,
    summarize_vcf_phasing_output_truth, VcfDomainStage, VcfImputationOutputTruthSummaryV1,
    VcfPhasingOutputTruthSummaryV1,
};
use serde::{Deserialize, Serialize};

pub(crate) const PHASING_IMPUTATION_TRUTH_FIXTURE_ID: &str = "phasing-imputation-truth";
pub(crate) const PHASING_IMPUTATION_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.phasing_imputation_truth.v1";
const PHASING_IMPUTATION_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.phasing_imputation_truth.expected.v1";
const PHASING_IMPUTATION_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.phasing_imputation_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PhasingImputationTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    expected_path: PathBuf,
    source_paths: Vec<PathBuf>,
    cases: Vec<PhasingImputationTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct PhasingImputationTruthCase {
    case_id: String,
    stage_id: String,
    tool_id: String,
    artifact_root_path: PathBuf,
    output_vcf_path: PathBuf,
    #[serde(default)]
    truth_vcf_path: Option<PathBuf>,
    truth_kind: PhasingImputationTruthKind,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum PhasingImputationTruthKind {
    PhasingOutput,
    ImputationOutput,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct PhasingImputationTruthValidationReport {
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
struct PhasingImputationTruthBundle {
    schema_version: String,
    fixture_id: String,
    cases: Vec<PhasingImputationTruthCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct PhasingImputationTruthCaseTruth {
    case_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    phasing_metrics: Option<VcfPhasingMetricsTruth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    impute_metrics: Option<VcfImputeMetricsTruth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    imputation_metrics: Option<VcfImputationMetricsTruth>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    phasing_output_truth: Option<VcfPhasingOutputTruthSummaryV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    imputation_output_truth: Option<VcfImputationOutputTruthSummaryV1>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfPhasingMetricsTruth {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    input_genotypes: u64,
    phased_genotypes: u64,
    unphased_genotypes: u64,
    phase_set_count: u64,
    sample_count: usize,
    sample_ids: Vec<String>,
    output_variant_count: u64,
    status: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfImputeMetricsTruth {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    variant_count: u64,
    missing_before: u64,
    missing_after: u64,
    imputed_genotypes: u64,
    low_confidence_count: u64,
    masked_truth_site_count: u64,
    masked_truth_match_count: u64,
    unresolved_count: u64,
    not_imputable_reasons: BTreeMap<String, u64>,
    sample_count: usize,
    sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct VcfImputationMetricsTruth {
    schema_version: String,
    stage_id: String,
    tool_id: String,
    status: String,
    mean_info_score: f64,
    r2_available: bool,
    low_confidence_sites: u64,
    masked_truth_sites: u64,
    missing_quality_fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    concordance: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    dosage_r2: Option<f64>,
    variant_count: u64,
    sample_count: usize,
    sample_ids: Vec<String>,
}

pub(crate) fn validate_phasing_imputation_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<PhasingImputationTruthValidationReport> {
    let manifest = load_manifest(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "phasing/imputation truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!(
            "phasing/imputation truth bundle is missing: {}",
            expected_path.display()
        ));
    }

    let expected = load_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    let expected_map = expected
        .cases
        .iter()
        .map(|case| (case.case_id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    let actual_map = actual
        .cases
        .iter()
        .map(|case| (case.case_id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    if expected_map.len() != actual_map.len() {
        return Err(anyhow!(
            "phasing/imputation truth case count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for case in &manifest.cases {
        let expected_case = expected_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!(
                "expected phasing/imputation truth is missing case `{}`",
                case.case_id
            )
        })?;
        let actual_case = actual_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!(
                "observed phasing/imputation truth is missing case `{}`",
                case.case_id
            )
        })?;
        if expected_case != actual_case {
            return Err(anyhow!(
                "phasing/imputation truth drifted for case `{}`\nexpected: {expected_case:#?}\nobserved: {actual_case:#?}",
                case.case_id
            ));
        }
    }

    Ok(PhasingImputationTruthValidationReport {
        schema_version: PHASING_IMPUTATION_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.cases.len(),
        validated_stage_ids: collect_stage_ids(&actual.cases),
        validated_tool_ids: collect_tool_ids(&actual.cases),
        valid: true,
    })
}

fn load_manifest(manifest_path: &Path) -> Result<PhasingImputationTruthManifest> {
    let raw =
        fs::read_to_string(manifest_path).with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &PhasingImputationTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != PHASING_IMPUTATION_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "phasing/imputation truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            PHASING_IMPUTATION_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != PHASING_IMPUTATION_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "phasing/imputation truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            PHASING_IMPUTATION_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "phasing/imputation truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "phasing/imputation truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "phasing/imputation truth source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "phasing/imputation truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }

    let mut case_ids = BTreeSet::new();
    for case in &manifest.cases {
        validate_case_contract(repo_root, case, manifest_path)?;
        if !case_ids.insert(case.case_id.clone()) {
            return Err(anyhow!(
                "phasing/imputation truth manifest repeats case_id `{}`",
                case.case_id
            ));
        }
    }
    Ok(())
}

fn validate_case_contract(
    repo_root: &Path,
    case: &PhasingImputationTruthCase,
    manifest_path: &Path,
) -> Result<()> {
    if case.case_id.trim().is_empty() || case.stage_id.trim().is_empty() || case.tool_id.trim().is_empty() {
        return Err(anyhow!(
            "phasing/imputation truth manifest `{}` contains a case with empty identity fields",
            manifest_path.display()
        ));
    }
    let artifact_root = resolve_repo_relative_path(repo_root, &case.artifact_root_path);
    if !artifact_root.is_dir() {
        return Err(anyhow!(
            "phasing/imputation truth case `{}` artifact root is missing: {}",
            case.case_id,
            artifact_root.display()
        ));
    }
    let output_vcf = resolve_repo_relative_path(repo_root, &case.output_vcf_path);
    if !output_vcf.is_file() {
        return Err(anyhow!(
            "phasing/imputation truth case `{}` output VCF is missing: {}",
            case.case_id,
            output_vcf.display()
        ));
    }
    let stage = parse_supported_stage(&case.stage_id)?;
    match (case.tool_id.as_str(), stage, case.truth_kind) {
        ("shapeit5" | "eagle" | "beagle", VcfDomainStage::Phasing, PhasingImputationTruthKind::PhasingOutput) => {
            if case.truth_vcf_path.is_some() {
                return Err(anyhow!(
                    "phasing/imputation truth case `{}` must not declare truth_vcf_path for phasing output",
                    case.case_id
                ));
            }
        }
        ("beagle" | "glimpse" | "impute5" | "minimac4", VcfDomainStage::Impute | VcfDomainStage::ImputationMetrics, PhasingImputationTruthKind::ImputationOutput) => {
            let truth_vcf_path = case.truth_vcf_path.as_ref().ok_or_else(|| {
                anyhow!(
                    "phasing/imputation truth case `{}` must declare truth_vcf_path for imputation output",
                    case.case_id
                )
            })?;
            let truth_vcf = resolve_repo_relative_path(repo_root, truth_vcf_path);
            if !truth_vcf.is_file() {
                return Err(anyhow!(
                    "phasing/imputation truth case `{}` truth VCF is missing: {}",
                    case.case_id,
                    truth_vcf.display()
                ));
            }
        }
        _ => {
            return Err(anyhow!(
                "phasing/imputation truth case `{}` uses unsupported tool/stage/truth combination `{}` / `{}` / {:?}",
                case.case_id,
                case.tool_id,
                case.stage_id,
                case.truth_kind
            ));
        }
    }
    Ok(())
}

fn load_bundle(expected_path: &Path) -> Result<PhasingImputationTruthBundle> {
    let raw =
        fs::read_to_string(expected_path).with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
    manifest: &PhasingImputationTruthManifest,
    bundle: &PhasingImputationTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != PHASING_IMPUTATION_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "phasing/imputation truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            PHASING_IMPUTATION_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "phasing/imputation truth bundle `{}` fixture_id `{}` must equal `{}`",
            expected_path.display(),
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let expected_case_ids = manifest
        .cases
        .iter()
        .map(|case| case.case_id.as_str())
        .collect::<BTreeSet<_>>();
    let bundle_case_ids = bundle
        .cases
        .iter()
        .map(|case| case.case_id.as_str())
        .collect::<BTreeSet<_>>();
    if expected_case_ids != bundle_case_ids {
        return Err(anyhow!(
            "phasing/imputation truth bundle case ids do not match manifest `{}`",
            expected_path.display()
        ));
    }
    Ok(())
}

fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &PhasingImputationTruthManifest,
) -> Result<PhasingImputationTruthBundle> {
    let mut cases = Vec::with_capacity(manifest.cases.len());
    for case in &manifest.cases {
        let stage = parse_supported_stage(&case.stage_id)?;
        let artifact_root = resolve_repo_relative_path(repo_root, &case.artifact_root_path);
        let output_vcf = resolve_repo_relative_path(repo_root, &case.output_vcf_path);
        let truth_vcf = case
            .truth_vcf_path
            .as_ref()
            .map(|path| resolve_repo_relative_path(repo_root, path));
        let case_truth = match stage {
            VcfDomainStage::Phasing => {
                let raw = parse_phasing_stage_metrics(&case.tool_id, &artifact_root)?;
                let mut output_truth =
                    summarize_vcf_phasing_output_truth(&output_vcf, &case.stage_id, &case.tool_id)?;
                output_truth.input_vcf = PathBuf::from(path_relative_to_repo(repo_root, &output_vcf));
                PhasingImputationTruthCaseTruth {
                    case_id: case.case_id.clone(),
                    phasing_metrics: Some(serde_json::from_value(raw).context("deserialize phasing truth metrics")?),
                    impute_metrics: None,
                    imputation_metrics: None,
                    phasing_output_truth: Some(output_truth),
                    imputation_output_truth: None,
                }
            }
            VcfDomainStage::Impute => {
                let raw = parse_imputation_stage_metrics(&case.tool_id, stage, &artifact_root)?;
                let mut output_truth = summarize_vcf_imputation_output_truth(
                    &output_vcf,
                    truth_vcf.as_deref(),
                    &case.stage_id,
                    &case.tool_id,
                )?;
                output_truth.input_vcf = PathBuf::from(path_relative_to_repo(repo_root, &output_vcf));
                output_truth.truth_vcf = truth_vcf
                    .as_ref()
                    .map(|path| PathBuf::from(path_relative_to_repo(repo_root, path)));
                PhasingImputationTruthCaseTruth {
                    case_id: case.case_id.clone(),
                    phasing_metrics: None,
                    impute_metrics: Some(serde_json::from_value(raw).context("deserialize impute truth metrics")?),
                    imputation_metrics: None,
                    phasing_output_truth: None,
                    imputation_output_truth: Some(output_truth),
                }
            }
            VcfDomainStage::ImputationMetrics => {
                let raw = parse_imputation_stage_metrics(&case.tool_id, stage, &artifact_root)?;
                let mut output_truth = summarize_vcf_imputation_output_truth(
                    &output_vcf,
                    truth_vcf.as_deref(),
                    &case.stage_id,
                    &case.tool_id,
                )?;
                output_truth.input_vcf = PathBuf::from(path_relative_to_repo(repo_root, &output_vcf));
                output_truth.truth_vcf = truth_vcf
                    .as_ref()
                    .map(|path| PathBuf::from(path_relative_to_repo(repo_root, path)));
                PhasingImputationTruthCaseTruth {
                    case_id: case.case_id.clone(),
                    phasing_metrics: None,
                    impute_metrics: None,
                    imputation_metrics: Some(
                        serde_json::from_value(raw).context("deserialize imputation-metrics truth")?,
                    ),
                    phasing_output_truth: None,
                    imputation_output_truth: Some(output_truth),
                }
            }
            other => {
                return Err(anyhow!(
                    "unsupported phasing/imputation truth stage `{}`",
                    other.as_str()
                ))
            }
        };
        cases.push(case_truth);
    }

    Ok(PhasingImputationTruthBundle {
        schema_version: PHASING_IMPUTATION_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        cases,
    })
}

fn parse_supported_stage(stage_id: &str) -> Result<VcfDomainStage> {
    match stage_id {
        "vcf.phasing" => Ok(VcfDomainStage::Phasing),
        "vcf.impute" => Ok(VcfDomainStage::Impute),
        "vcf.imputation_metrics" => Ok(VcfDomainStage::ImputationMetrics),
        other => Err(anyhow!("unsupported phasing/imputation truth stage `{other}`")),
    }
}

fn collect_stage_ids(cases: &[PhasingImputationTruthCaseTruth]) -> Vec<String> {
    let mut values = cases
        .iter()
        .map(|case| {
            case.phasing_metrics
                .as_ref()
                .map(|metrics| metrics.stage_id.clone())
                .or_else(|| case.impute_metrics.as_ref().map(|metrics| metrics.stage_id.clone()))
                .or_else(|| {
                    case.imputation_metrics
                        .as_ref()
                        .map(|metrics| metrics.stage_id.clone())
                })
                .expect("case truth must carry metrics")
        })
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn collect_tool_ids(cases: &[PhasingImputationTruthCaseTruth]) -> Vec<String> {
    let mut values = cases
        .iter()
        .map(|case| {
            case.phasing_metrics
                .as_ref()
                .map(|metrics| metrics.tool_id.clone())
                .or_else(|| case.impute_metrics.as_ref().map(|metrics| metrics.tool_id.clone()))
                .or_else(|| {
                    case.imputation_metrics
                        .as_ref()
                        .map(|metrics| metrics.tool_id.clone())
                })
                .expect("case truth must carry metrics")
        })
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn resolve_repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() { path.to_path_buf() } else { repo_root.join(path) }
}

fn resolve_fixture_path(fixture_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() { path.to_path_buf() } else { fixture_root.join(path) }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
