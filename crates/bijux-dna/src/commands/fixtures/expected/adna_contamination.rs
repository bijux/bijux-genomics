#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::metrics::{BamMetricsV1, ContaminationMetricsV1};
use bijux_dna_domain_bam::{
    inspect_tiny_alignment, summarize_bam_adna_contamination_truth,
    BamAdnaContaminationTruthSummaryV1, BAM_ADNA_CONTAMINATION_TRUTH_SUMMARY_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};

pub(crate) const ADNA_CONTAMINATION_TRUTH_FIXTURE_ID: &str = "adna-contamination-truth";
pub(crate) const ADNA_CONTAMINATION_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.adna_contamination_truth.v1";
const ADNA_CONTAMINATION_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.adna_contamination_truth.expected.v1";
const ADNA_CONTAMINATION_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.adna_contamination_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdnaContaminationTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    expected_path: PathBuf,
    source_paths: Vec<PathBuf>,
    cases: Vec<AdnaContaminationTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdnaContaminationTruthCase {
    case_id: String,
    sample_id: String,
    cohort: String,
    tool: String,
    alignment_path: PathBuf,
    reference_path: PathBuf,
    minimum_mean_coverage: f64,
    observed_mean_coverage: f64,
    raw_estimate: f64,
    raw_ci_low: f64,
    raw_ci_high: f64,
    #[serde(default, flatten)]
    context: AdnaContaminationTruthCaseContextSignals,
    #[serde(default, flatten)]
    compatibility: AdnaContaminationTruthCaseCompatibilitySignals,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdnaContaminationTruthCaseContextSignals {
    #[serde(default)]
    has_mito_reference: bool,
    #[serde(default)]
    has_damage_context: bool,
    #[serde(default)]
    has_reference_panel: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdnaContaminationTruthCaseCompatibilitySignals {
    #[serde(default)]
    panel_build_compatible: bool,
    #[serde(default)]
    sex_context_available: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdnaContaminationTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_case_count: usize,
    pub(crate) validated_insufficient_case_count: usize,
    pub(crate) tool_ids: Vec<String>,
    pub(crate) checked_cases: Vec<String>,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct AdnaContaminationTruthBundle {
    schema_version: String,
    fixture_id: String,
    truths: Vec<AdnaContaminationSampleTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct AdnaContaminationSampleTruth {
    case_id: String,
    sample_id: String,
    cohort: String,
    tool: String,
    alignment_path: String,
    reference_path: String,
    summary: BamAdnaContaminationTruthSummaryV1,
}

pub(crate) fn validate_adna_contamination_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<AdnaContaminationTruthValidationReport> {
    let manifest = load_adna_contamination_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "aDNA contamination truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!(
            "aDNA contamination truth bundle is missing: {}",
            expected_path.display()
        ));
    }

    let expected = load_adna_contamination_truth_bundle(&expected_path)?;
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
            "aDNA contamination truth case count drifted: expected {}, observed {}",
            expected_truths.len(),
            actual_truths.len()
        ));
    }
    for case in &manifest.cases {
        let expected_truth = expected_truths.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("expected aDNA contamination truth is missing case `{}`", case.case_id)
        })?;
        let actual_truth = actual_truths.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("observed aDNA contamination truth is missing case `{}`", case.case_id)
        })?;
        if expected_truth != actual_truth {
            return Err(anyhow!("aDNA contamination truth drifted for case `{}`", case.case_id));
        }
    }

    let tool_ids = actual
        .truths
        .iter()
        .map(|truth| truth.tool.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let validated_insufficient_case_count =
        actual.truths.iter().filter(|truth| truth.summary.status == "insufficient").count();

    Ok(AdnaContaminationTruthValidationReport {
        schema_version: ADNA_CONTAMINATION_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.truths.len(),
        validated_insufficient_case_count,
        tool_ids,
        checked_cases: manifest.cases.iter().map(|case| case.case_id.clone()).collect(),
        valid: true,
    })
}

fn load_adna_contamination_truth_manifest_path(
    manifest_path: &Path,
) -> Result<AdnaContaminationTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn load_adna_contamination_truth_bundle(
    expected_path: &Path,
) -> Result<AdnaContaminationTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &AdnaContaminationTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != ADNA_CONTAMINATION_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "aDNA contamination truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            ADNA_CONTAMINATION_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != ADNA_CONTAMINATION_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "aDNA contamination truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            ADNA_CONTAMINATION_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for path in &manifest.source_paths {
        let absolute = resolve_repo_relative_path(repo_root, path);
        if !absolute.exists() {
            return Err(anyhow!(
                "aDNA contamination truth manifest source path is missing: {}",
                absolute.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }

    let mut case_ids = BTreeSet::new();
    for case in &manifest.cases {
        validate_adna_contamination_case_contract(repo_root, manifest_path, &mut case_ids, case)?;
    }

    Ok(())
}

fn validate_adna_contamination_case_contract(
    repo_root: &Path,
    manifest_path: &Path,
    case_ids: &mut BTreeSet<String>,
    case: &AdnaContaminationTruthCase,
) -> Result<()> {
    if case.case_id.trim().is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth manifest `{}` contains an empty case_id",
            manifest_path.display()
        ));
    }
    if !case_ids.insert(case.case_id.clone()) {
        return Err(anyhow!(
            "aDNA contamination truth manifest repeats case_id `{}`",
            case.case_id
        ));
    }
    if case.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must declare a non-empty sample_id",
            case.case_id
        ));
    }
    if case.cohort.trim().is_empty() {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must declare a non-empty cohort",
            case.case_id
        ));
    }
    if case.tool != "schmutzi" && case.tool != "verifybamid2" && case.tool != "contammix" {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must use schmutzi, verifybamid2, or contammix",
            case.case_id
        ));
    }
    if case.minimum_mean_coverage < 0.0 || case.observed_mean_coverage < 0.0 {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must keep coverage non-negative",
            case.case_id
        ));
    }
    if !(0.0..=1.0).contains(&case.raw_estimate)
        || !(0.0..=1.0).contains(&case.raw_ci_low)
        || !(0.0..=1.0).contains(&case.raw_ci_high)
    {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must keep contamination values within [0, 1]",
            case.case_id
        ));
    }
    if case.raw_ci_low > case.raw_estimate || case.raw_estimate > case.raw_ci_high {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` must keep ci_low <= estimate <= ci_high",
            case.case_id
        ));
    }

    let alignment_path = resolve_repo_relative_path(repo_root, &case.alignment_path);
    if !alignment_path.is_file() {
        return Err(anyhow!(
            "aDNA contamination truth alignment path is missing for case `{}`: {}",
            case.case_id,
            alignment_path.display()
        ));
    }
    let reference_path = resolve_repo_relative_path(repo_root, &case.reference_path);
    if !reference_path.is_file() {
        return Err(anyhow!(
            "aDNA contamination truth reference path is missing for case `{}`: {}",
            case.case_id,
            reference_path.display()
        ));
    }

    Ok(())
}

fn validate_bundle_contract(
    manifest: &AdnaContaminationTruthManifest,
    bundle: &AdnaContaminationTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != ADNA_CONTAMINATION_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "aDNA contamination truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            ADNA_CONTAMINATION_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "aDNA contamination truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }

    let expected_case_ids =
        manifest.cases.iter().map(|case| case.case_id.as_str()).collect::<BTreeSet<_>>();
    let observed_case_ids =
        bundle.truths.iter().map(|truth| truth.case_id.as_str()).collect::<BTreeSet<_>>();
    if expected_case_ids != observed_case_ids {
        return Err(anyhow!(
            "aDNA contamination truth bundle `{}` must contain cases {:?}",
            expected_path.display(),
            expected_case_ids
        ));
    }
    for truth in &bundle.truths {
        if truth.case_id.trim().is_empty()
            || truth.sample_id.trim().is_empty()
            || truth.cohort.trim().is_empty()
            || truth.tool.trim().is_empty()
        {
            return Err(anyhow!(
                "aDNA contamination truth bundle `{}` contains an incomplete case header",
                expected_path.display()
            ));
        }
        if truth.alignment_path.trim().is_empty() || truth.reference_path.trim().is_empty() {
            return Err(anyhow!(
                "aDNA contamination truth case `{}` must declare non-empty input paths",
                truth.case_id
            ));
        }
        if truth.summary.schema_version != BAM_ADNA_CONTAMINATION_TRUTH_SUMMARY_SCHEMA_VERSION {
            return Err(anyhow!(
                "aDNA contamination truth case `{}` uses schema `{}` instead of `{}`",
                truth.case_id,
                truth.summary.schema_version,
                BAM_ADNA_CONTAMINATION_TRUTH_SUMMARY_SCHEMA_VERSION
            ));
        }
        if truth.summary.stage_id != "bam.contamination" {
            return Err(anyhow!(
                "aDNA contamination truth case `{}` must resolve to stage_id `bam.contamination`",
                truth.case_id
            ));
        }
    }

    Ok(())
}

fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &AdnaContaminationTruthManifest,
) -> Result<AdnaContaminationTruthBundle> {
    let truths = manifest
        .cases
        .iter()
        .map(|case| build_actual_case_truth(repo_root, case))
        .collect::<Result<Vec<_>>>()?;
    Ok(AdnaContaminationTruthBundle {
        schema_version: ADNA_CONTAMINATION_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        truths,
    })
}

fn build_actual_case_truth(
    repo_root: &Path,
    case: &AdnaContaminationTruthCase,
) -> Result<AdnaContaminationSampleTruth> {
    let alignment_path = resolve_repo_relative_path(repo_root, &case.alignment_path);
    let reference_path = resolve_repo_relative_path(repo_root, &case.reference_path);
    let inspection = inspect_tiny_alignment(&alignment_path)?;
    if !inspection.header_sample_ids.iter().any(|sample_id| sample_id == &case.sample_id) {
        return Err(anyhow!(
            "aDNA contamination truth case `{}` sample_id `{}` is not present in alignment header samples",
            case.case_id,
            case.sample_id
        ));
    }

    let mut metrics = BamMetricsV1::empty();
    metrics.coverage.mean = case.observed_mean_coverage;
    metrics.contamination = ContaminationMetricsV1 {
        method: case.tool.clone(),
        estimate: case.raw_estimate,
        ci_low: case.raw_ci_low,
        ci_high: case.raw_ci_high,
        assumptions: vec![
            format!("tool:{}", case.tool),
            format!("case:{}", case.case_id),
            format!("sample:{}", case.sample_id),
        ],
    };

    let summary = summarize_bam_adna_contamination_truth(
        &case.tool,
        &metrics,
        case.minimum_mean_coverage,
        case.context.has_mito_reference,
        case.context.has_damage_context,
        case.context.has_reference_panel,
        case.compatibility.panel_build_compatible,
        case.compatibility.sex_context_available,
    )?;

    Ok(AdnaContaminationSampleTruth {
        case_id: case.case_id.clone(),
        sample_id: case.sample_id.clone(),
        cohort: case.cohort.clone(),
        tool: case.tool.clone(),
        alignment_path: path_relative_to_repo(repo_root, &alignment_path),
        reference_path: path_relative_to_repo(repo_root, &reference_path),
        summary,
    })
}

fn resolve_fixture_path(fixture_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        fixture_root.join(path)
    }
}

fn resolve_repo_relative_path(repo_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        repo_root.join(path)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root)
        .map_or_else(|_| path.display().to_string(), |relative| relative.display().to_string())
}
