#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::metrics::DamageMetricsV1;
use bijux_dna_domain_bam::params::UdgModel;
use bijux_dna_domain_bam::{
    inspect_tiny_alignment, summarize_tiny_bam_adna_damage_truth, BamAdnaDamageTruthSummaryV1,
    BAM_ADNA_DAMAGE_TRUTH_SUMMARY_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};

pub(crate) const ADNA_DAMAGE_TRUTH_FIXTURE_ID: &str = "adna-damage-truth";
pub(crate) const ADNA_DAMAGE_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.adna_damage_truth.v1";
const ADNA_DAMAGE_TRUTH_BUNDLE_SCHEMA_VERSION: &str = "bijux.bench.adna_damage_truth.expected.v1";
const ADNA_DAMAGE_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.adna_damage_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdnaDamageTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    expected_path: PathBuf,
    source_paths: Vec<PathBuf>,
    cases: Vec<AdnaDamageTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct AdnaDamageTruthCase {
    case_id: String,
    sample_id: String,
    cohort: String,
    alignment_path: PathBuf,
    reference_path: PathBuf,
    udg_model: UdgModel,
    strict_profile: bool,
    terminal_c_to_t_5p: f64,
    terminal_g_to_a_3p: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AdnaDamageTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_case_count: usize,
    pub(crate) validated_insufficient_case_count: usize,
    pub(crate) terminal_pattern_classes: Vec<String>,
    pub(crate) checked_cases: Vec<String>,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct AdnaDamageTruthBundle {
    schema_version: String,
    fixture_id: String,
    truths: Vec<AdnaDamageSampleTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct AdnaDamageSampleTruth {
    case_id: String,
    sample_id: String,
    cohort: String,
    alignment_path: String,
    reference_path: String,
    summary: BamAdnaDamageTruthSummaryV1,
}

pub(crate) fn validate_adna_damage_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<AdnaDamageTruthValidationReport> {
    let manifest = load_adna_damage_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("aDNA damage truth manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("aDNA damage truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_adna_damage_truth_bundle(&expected_path)?;
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
            "aDNA damage truth case count drifted: expected {}, observed {}",
            expected_truths.len(),
            actual_truths.len()
        ));
    }
    for case in &manifest.cases {
        let expected_truth = expected_truths.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("expected aDNA damage truth is missing case `{}`", case.case_id)
        })?;
        let actual_truth = actual_truths.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("observed aDNA damage truth is missing case `{}`", case.case_id)
        })?;
        if expected_truth != actual_truth {
            return Err(anyhow!("aDNA damage truth drifted for case `{}`", case.case_id));
        }
    }

    let terminal_pattern_classes = actual
        .truths
        .iter()
        .map(|truth| truth.summary.terminal_pattern_class.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let validated_insufficient_case_count =
        actual.truths.iter().filter(|truth| truth.summary.insufficiency_reason.is_some()).count();

    Ok(AdnaDamageTruthValidationReport {
        schema_version: ADNA_DAMAGE_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.truths.len(),
        validated_insufficient_case_count,
        terminal_pattern_classes,
        checked_cases: manifest.cases.iter().map(|case| case.case_id.clone()).collect(),
        valid: true,
    })
}

fn load_adna_damage_truth_manifest_path(manifest_path: &Path) -> Result<AdnaDamageTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn load_adna_damage_truth_bundle(expected_path: &Path) -> Result<AdnaDamageTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &AdnaDamageTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != ADNA_DAMAGE_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "aDNA damage truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            ADNA_DAMAGE_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != ADNA_DAMAGE_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "aDNA damage truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            ADNA_DAMAGE_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "aDNA damage truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "aDNA damage truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for path in &manifest.source_paths {
        let absolute = resolve_repo_relative_path(repo_root, path);
        if !absolute.is_file() {
            return Err(anyhow!(
                "aDNA damage truth manifest source path is missing: {}",
                absolute.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "aDNA damage truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }

    let mut case_ids = BTreeSet::new();
    for case in &manifest.cases {
        if case.case_id.trim().is_empty() {
            return Err(anyhow!(
                "aDNA damage truth manifest `{}` contains an empty case_id",
                manifest_path.display()
            ));
        }
        if !case_ids.insert(case.case_id.clone()) {
            return Err(anyhow!("aDNA damage truth manifest repeats case_id `{}`", case.case_id));
        }
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "aDNA damage truth case `{}` must declare a non-empty sample_id",
                case.case_id
            ));
        }
        if case.cohort.trim().is_empty() {
            return Err(anyhow!(
                "aDNA damage truth case `{}` must declare a non-empty cohort",
                case.case_id
            ));
        }
        if !(0.0..=1.0).contains(&case.terminal_c_to_t_5p) {
            return Err(anyhow!(
                "aDNA damage truth case `{}` must keep terminal_c_to_t_5p within [0, 1]",
                case.case_id
            ));
        }
        if !(0.0..=1.0).contains(&case.terminal_g_to_a_3p) {
            return Err(anyhow!(
                "aDNA damage truth case `{}` must keep terminal_g_to_a_3p within [0, 1]",
                case.case_id
            ));
        }

        let alignment_path = resolve_repo_relative_path(repo_root, &case.alignment_path);
        if !alignment_path.is_file() {
            return Err(anyhow!(
                "aDNA damage truth alignment path is missing for case `{}`: {}",
                case.case_id,
                alignment_path.display()
            ));
        }
        let reference_path = resolve_repo_relative_path(repo_root, &case.reference_path);
        if !reference_path.is_file() {
            return Err(anyhow!(
                "aDNA damage truth reference path is missing for case `{}`: {}",
                case.case_id,
                reference_path.display()
            ));
        }
    }

    Ok(())
}

fn validate_bundle_contract(
    manifest: &AdnaDamageTruthManifest,
    bundle: &AdnaDamageTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != ADNA_DAMAGE_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "aDNA damage truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            ADNA_DAMAGE_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "aDNA damage truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
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
            "aDNA damage truth bundle `{}` must contain cases {:?}",
            expected_path.display(),
            expected_case_ids
        ));
    }
    for truth in &bundle.truths {
        if truth.case_id.trim().is_empty() {
            return Err(anyhow!(
                "aDNA damage truth bundle `{}` contains an empty case_id",
                expected_path.display()
            ));
        }
        if truth.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "aDNA damage truth case `{}` must declare a non-empty sample_id",
                truth.case_id
            ));
        }
        if truth.cohort.trim().is_empty() {
            return Err(anyhow!(
                "aDNA damage truth case `{}` must declare a non-empty cohort",
                truth.case_id
            ));
        }
        if truth.alignment_path.trim().is_empty() || truth.reference_path.trim().is_empty() {
            return Err(anyhow!(
                "aDNA damage truth case `{}` must declare non-empty input paths",
                truth.case_id
            ));
        }
        if truth.summary.schema_version != BAM_ADNA_DAMAGE_TRUTH_SUMMARY_SCHEMA_VERSION {
            return Err(anyhow!(
                "aDNA damage truth case `{}` uses schema `{}` instead of `{}`",
                truth.case_id,
                truth.summary.schema_version,
                BAM_ADNA_DAMAGE_TRUTH_SUMMARY_SCHEMA_VERSION
            ));
        }
        if truth.summary.stage_id != "bam.damage" {
            return Err(anyhow!(
                "aDNA damage truth case `{}` must resolve to stage_id `bam.damage`",
                truth.case_id
            ));
        }
    }

    Ok(())
}

fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &AdnaDamageTruthManifest,
) -> Result<AdnaDamageTruthBundle> {
    let truths = manifest
        .cases
        .iter()
        .map(|case| build_actual_case_truth(repo_root, case))
        .collect::<Result<Vec<_>>>()?;
    Ok(AdnaDamageTruthBundle {
        schema_version: ADNA_DAMAGE_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: manifest.fixture_id.clone(),
        truths,
    })
}

fn build_actual_case_truth(
    repo_root: &Path,
    case: &AdnaDamageTruthCase,
) -> Result<AdnaDamageSampleTruth> {
    let alignment_path = resolve_repo_relative_path(repo_root, &case.alignment_path);
    let reference_path = resolve_repo_relative_path(repo_root, &case.reference_path);
    let inspection = inspect_tiny_alignment(&alignment_path)?;
    if !inspection.header_sample_ids.iter().any(|sample_id| sample_id == &case.sample_id) {
        return Err(anyhow!(
            "aDNA damage truth case `{}` sample_id `{}` is not present in alignment header samples",
            case.case_id,
            case.sample_id
        ));
    }
    let summary = summarize_tiny_bam_adna_damage_truth(
        &alignment_path,
        &DamageMetricsV1 {
            c_to_t_5p: case.terminal_c_to_t_5p,
            g_to_a_3p: case.terminal_g_to_a_3p,
            pmd_score_histogram: Vec::new(),
        },
        case.strict_profile,
        case.udg_model,
    )?;

    Ok(AdnaDamageSampleTruth {
        case_id: case.case_id.clone(),
        sample_id: case.sample_id.clone(),
        cohort: case.cohort.clone(),
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
