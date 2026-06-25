#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::{
    summarize_tiny_bam_haplogroup_truth, BamHaplogroupTruthSummaryV1,
    BAM_HAPLOGROUP_TRUTH_SUMMARY_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_corpus_fixture::bam::{
    validate_bam_corpus_fixture_manifest_path, BamCorpusFixtureManifest, BamCorpusFixtureSample,
};

pub(crate) const BAM_HAPLOGROUP_TRUTH_FIXTURE_ID: &str = "haplogroup-truth";
pub(crate) const BAM_HAPLOGROUP_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.bam_haplogroup_truth.v1";
const BAM_HAPLOGROUP_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.bam_haplogroup_truth.expected.v1";
const BAM_HAPLOGROUP_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_haplogroup_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BamHaplogroupTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    corpus_manifest_path: PathBuf,
    expected_path: PathBuf,
    source_paths: Vec<PathBuf>,
    cases: Vec<BamHaplogroupTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BamHaplogroupTruthCase {
    case_id: String,
    sample_id: String,
    method: String,
    reference_panel_id: String,
    reference_build: String,
    population_scope: String,
    minimum_coverage: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamHaplogroupTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_case_count: usize,
    pub(crate) validated_ready_case_count: usize,
    pub(crate) validated_uncertain_case_count: usize,
    pub(crate) validated_coverage_gate_case_count: usize,
    pub(crate) validated_statuses: Vec<String>,
    pub(crate) checked_cases: Vec<String>,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BamHaplogroupTruthBundle {
    schema_version: String,
    fixture_id: String,
    case_truths: Vec<BamHaplogroupCaseTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BamHaplogroupCaseTruth {
    case_id: String,
    sample_id: String,
    cohort: String,
    alignment_path: String,
    reference_panel_path: String,
    summary: BamHaplogroupTruthSummary,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BamHaplogroupTruthSummary {
    summary_schema_version: String,
    stage_id: String,
    method: String,
    reference_panel_id: String,
    reference_build: String,
    population_scope: String,
    minimum_coverage: f64,
    observed_mean_coverage: f64,
    ready: bool,
    #[serde(default)]
    haplogroup_call: Option<String>,
    confidence: f64,
    status: String,
    markers_total: u64,
    markers_supported: u64,
    #[serde(default)]
    supported_marker_ids: Vec<String>,
    #[serde(default)]
    lineage_scope: Option<String>,
    #[serde(default)]
    refusal_codes: Vec<String>,
}

pub(crate) fn validate_bam_haplogroup_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamHaplogroupTruthValidationReport> {
    let manifest = load_bam_haplogroup_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM haplogroup truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("BAM haplogroup truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_bam_haplogroup_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    let expected_map = expected
        .case_truths
        .iter()
        .map(|case| (case.case_id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    let actual_map = actual
        .case_truths
        .iter()
        .map(|case| (case.case_id.as_str(), case))
        .collect::<BTreeMap<_, _>>();
    if expected_map.len() != actual_map.len() {
        return Err(anyhow!(
            "BAM haplogroup truth case count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for case in &manifest.cases {
        let expected_case = expected_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("expected BAM haplogroup truth is missing case `{}`", case.case_id)
        })?;
        let actual_case = actual_map.get(case.case_id.as_str()).ok_or_else(|| {
            anyhow!("observed BAM haplogroup truth is missing case `{}`", case.case_id)
        })?;
        if expected_case != actual_case {
            return Err(anyhow!(
                "BAM haplogroup truth drifted for case `{}`\nexpected: {expected_case:#?}\nobserved: {actual_case:#?}",
                case.case_id
            ));
        }
    }

    let validated_statuses = collect_statuses(&actual.case_truths);
    validate_required_statuses(&validated_statuses, manifest_path)?;

    Ok(BamHaplogroupTruthValidationReport {
        schema_version: BAM_HAPLOGROUP_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.case_truths.len(),
        validated_ready_case_count: actual
            .case_truths
            .iter()
            .filter(|case| case.summary.status == "ready")
            .count(),
        validated_uncertain_case_count: actual
            .case_truths
            .iter()
            .filter(|case| case.summary.status == "uncertain_marker_support")
            .count(),
        validated_coverage_gate_case_count: actual
            .case_truths
            .iter()
            .filter(|case| case.summary.status == "coverage_gate_not_met")
            .count(),
        validated_statuses,
        checked_cases: manifest.cases.iter().map(|case| case.case_id.clone()).collect(),
        valid: true,
    })
}

fn load_bam_haplogroup_truth_manifest_path(
    manifest_path: &Path,
) -> Result<BamHaplogroupTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &BamHaplogroupTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != BAM_HAPLOGROUP_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM haplogroup truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            BAM_HAPLOGROUP_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != BAM_HAPLOGROUP_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "BAM haplogroup truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            BAM_HAPLOGROUP_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "BAM haplogroup truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    if !corpus_manifest_path.is_file() {
        return Err(anyhow!(
            "BAM haplogroup truth corpus manifest is missing: {}",
            corpus_manifest_path.display()
        ));
    }
    validate_bam_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "BAM haplogroup truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "BAM haplogroup truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "BAM haplogroup truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }

    let mut case_ids = BTreeSet::new();
    for case in &manifest.cases {
        if case.case_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM haplogroup truth manifest `{}` contains an empty case_id",
                manifest_path.display()
            ));
        }
        if !case_ids.insert(case.case_id.clone()) {
            return Err(anyhow!(
                "BAM haplogroup truth manifest repeats case_id `{}`",
                case.case_id
            ));
        }
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM haplogroup truth case `{}` must declare a non-empty sample_id",
                case.case_id
            ));
        }
        if case.method.trim().is_empty()
            || case.reference_panel_id.trim().is_empty()
            || case.reference_build.trim().is_empty()
            || case.population_scope.trim().is_empty()
        {
            return Err(anyhow!(
                "BAM haplogroup truth case `{}` must declare non-empty method, panel, build, and population scope",
                case.case_id
            ));
        }
        if case.minimum_coverage < 0.0 {
            return Err(anyhow!(
                "BAM haplogroup truth case `{}` must keep non-negative minimum_coverage",
                case.case_id
            ));
        }
    }

    Ok(())
}

fn load_bam_haplogroup_truth_bundle(expected_path: &Path) -> Result<BamHaplogroupTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
    manifest: &BamHaplogroupTruthManifest,
    bundle: &BamHaplogroupTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != BAM_HAPLOGROUP_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM haplogroup truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            BAM_HAPLOGROUP_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM haplogroup truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let actual_case_ids =
        bundle.case_truths.iter().map(|case| case.case_id.as_str()).collect::<Vec<_>>();
    let expected_case_ids =
        manifest.cases.iter().map(|case| case.case_id.as_str()).collect::<Vec<_>>();
    if actual_case_ids != expected_case_ids {
        return Err(anyhow!(
            "BAM haplogroup truth bundle `{}` must contain cases {:?}",
            expected_path.display(),
            expected_case_ids
        ));
    }
    for case_truth in &bundle.case_truths {
        validate_case_truth_contract(case_truth, expected_path)?;
    }
    validate_required_statuses(&collect_statuses(&bundle.case_truths), expected_path)?;
    Ok(())
}

fn validate_case_truth_contract(
    case_truth: &BamHaplogroupCaseTruth,
    expected_path: &Path,
) -> Result<()> {
    validate_case_truth_identity(case_truth, expected_path)?;
    validate_case_truth_summary(case_truth)?;
    validate_case_truth_status(case_truth)?;
    Ok(())
}

fn validate_case_truth_identity(
    case_truth: &BamHaplogroupCaseTruth,
    expected_path: &Path,
) -> Result<()> {
    if case_truth.case_id.trim().is_empty() || case_truth.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM haplogroup truth bundle `{}` contains a case with an empty case_id or sample_id",
            expected_path.display()
        ));
    }
    if case_truth.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must declare a non-empty cohort",
            case_truth.case_id
        ));
    }
    if case_truth.alignment_path.trim().is_empty()
        || case_truth.reference_panel_path.trim().is_empty()
    {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must declare non-empty input paths",
            case_truth.case_id
        ));
    }
    if case_truth.summary.summary_schema_version != BAM_HAPLOGROUP_TRUTH_SUMMARY_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` uses schema `{}` instead of `{}`",
            case_truth.case_id,
            case_truth.summary.summary_schema_version,
            BAM_HAPLOGROUP_TRUTH_SUMMARY_SCHEMA_VERSION
        ));
    }
    if case_truth.summary.stage_id != "bam.haplogroups" {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must keep stage_id `bam.haplogroups`",
            case_truth.case_id
        ));
    }
    Ok(())
}

fn validate_case_truth_summary(case_truth: &BamHaplogroupCaseTruth) -> Result<()> {
    if case_truth.summary.method.trim().is_empty()
        || case_truth.summary.reference_panel_id.trim().is_empty()
        || case_truth.summary.reference_build.trim().is_empty()
        || case_truth.summary.population_scope.trim().is_empty()
        || case_truth.summary.status.trim().is_empty()
    {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must declare non-empty method, panel, build, population scope, and status",
            case_truth.case_id
        ));
    }
    if case_truth.summary.minimum_coverage < 0.0
        || case_truth.summary.observed_mean_coverage < 0.0
        || case_truth.summary.confidence < 0.0
        || case_truth.summary.confidence > 1.0
    {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must keep non-negative coverage values and confidence within [0, 1]",
            case_truth.case_id
        ));
    }
    if case_truth.summary.markers_total == 0
        || case_truth.summary.markers_supported > case_truth.summary.markers_total
    {
        return Err(anyhow!(
            "BAM haplogroup truth case `{}` must keep marker counts consistent",
            case_truth.case_id
        ));
    }
    Ok(())
}

fn validate_case_truth_status(case_truth: &BamHaplogroupCaseTruth) -> Result<()> {
    match case_truth.summary.status.as_str() {
        "ready" => {
            if !case_truth.summary.ready
                || case_truth.summary.haplogroup_call.is_none()
                || !case_truth.summary.refusal_codes.is_empty()
            {
                return Err(anyhow!(
                    "BAM haplogroup truth case `{}` must keep a ready haplogroup call without refusal codes",
                    case_truth.case_id
                ));
            }
        }
        "uncertain_marker_support" => {
            if case_truth.summary.ready
                || case_truth.summary.haplogroup_call.is_none()
                || !case_truth
                    .summary
                    .refusal_codes
                    .iter()
                    .any(|code| code == "marker_support_incomplete")
            {
                return Err(anyhow!(
                    "BAM haplogroup truth case `{}` must keep partial marker support semantics",
                    case_truth.case_id
                ));
            }
        }
        "coverage_gate_not_met" => {
            if case_truth.summary.ready
                || case_truth.summary.haplogroup_call.is_some()
                || case_truth.summary.confidence != 0.0
                || !case_truth
                    .summary
                    .refusal_codes
                    .iter()
                    .any(|code| code == "coverage_below_haplogroup_minimum")
            {
                return Err(anyhow!(
                    "BAM haplogroup truth case `{}` must keep coverage-gated refusal semantics",
                    case_truth.case_id
                ));
            }
        }
        other => {
            return Err(anyhow!(
                "BAM haplogroup truth case `{}` has unsupported status `{other}`",
                case_truth.case_id
            ));
        }
    }
    Ok(())
}

fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &BamHaplogroupTruthManifest,
) -> Result<BamHaplogroupTruthBundle> {
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    let corpus = load_bam_corpus_fixture_manifest_path(&corpus_manifest_path)?;
    let manifest_dir = corpus_manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM corpus fixture manifest has no parent directory: {}",
            corpus_manifest_path.display()
        )
    })?;

    let case_truths = manifest
        .cases
        .iter()
        .map(|case| {
            let sample = corpus
                .samples
                .iter()
                .find(|sample| sample.sample_id == case.sample_id)
                .ok_or_else(|| {
                    anyhow!("BAM corpus fixture is missing sample `{}`", case.sample_id)
                })?;
            build_case_truth(repo_root, manifest_dir, sample, case)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BamHaplogroupTruthBundle {
        schema_version: BAM_HAPLOGROUP_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: BAM_HAPLOGROUP_TRUTH_FIXTURE_ID.to_string(),
        case_truths,
    })
}

fn load_bam_corpus_fixture_manifest_path(manifest_path: &Path) -> Result<BamCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn build_case_truth(
    repo_root: &Path,
    manifest_dir: &Path,
    sample: &BamCorpusFixtureSample,
    case: &BamHaplogroupTruthCase,
) -> Result<BamHaplogroupCaseTruth> {
    let alignment_path = resolve_manifest_relative_path(manifest_dir, &sample.alignment_path);
    let reference_panel_path = sample
        .source_paths
        .iter()
        .map(|path| resolve_repo_relative_path(repo_root, path))
        .find(|path| path.extension().and_then(|value| value.to_str()) == Some("tsv"))
        .ok_or_else(|| {
            anyhow!(
                "BAM haplogroup sample `{}` must expose a reference panel TSV in source_paths",
                sample.sample_id
            )
        })?;
    let summary = summarize_tiny_bam_haplogroup_truth(
        &alignment_path,
        &case.method,
        &reference_panel_path,
        &case.reference_panel_id,
        &case.reference_build,
        &case.population_scope,
        case.minimum_coverage,
    )?;

    Ok(BamHaplogroupCaseTruth {
        case_id: case.case_id.clone(),
        sample_id: sample.sample_id.clone(),
        cohort: sample.cohort.clone(),
        alignment_path: path_relative_to_repo(repo_root, &alignment_path),
        reference_panel_path: path_relative_to_repo(repo_root, &reference_panel_path),
        summary: normalize_haplogroup_summary(summary),
    })
}

fn normalize_haplogroup_summary(summary: BamHaplogroupTruthSummaryV1) -> BamHaplogroupTruthSummary {
    BamHaplogroupTruthSummary {
        summary_schema_version: summary.schema_version,
        stage_id: summary.stage_id,
        method: summary.method,
        reference_panel_id: summary.reference_panel_id,
        reference_build: summary.reference_build,
        population_scope: summary.population_scope,
        minimum_coverage: summary.minimum_coverage,
        observed_mean_coverage: summary.observed_mean_coverage,
        ready: summary.ready,
        haplogroup_call: summary.haplogroup_call,
        confidence: summary.confidence,
        status: summary.status,
        markers_total: summary.markers_total,
        markers_supported: summary.markers_supported,
        supported_marker_ids: summary.supported_marker_ids,
        lineage_scope: summary.lineage_scope,
        refusal_codes: summary.refusal_codes,
    }
}

fn validate_required_statuses(statuses: &[String], path: &Path) -> Result<()> {
    let required = ["coverage_gate_not_met", "ready", "uncertain_marker_support"];
    for status in required {
        if !statuses.iter().any(|value| value == status) {
            return Err(anyhow!(
                "BAM haplogroup truth `{}` must cover the `{status}` status",
                path.display()
            ));
        }
    }
    Ok(())
}

fn collect_statuses(case_truths: &[BamHaplogroupCaseTruth]) -> Vec<String> {
    let mut statuses =
        case_truths.iter().map(|case| case.summary.status.clone()).collect::<Vec<_>>();
    statuses.sort();
    statuses.dedup();
    statuses
}

fn resolve_fixture_path(fixture_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        fixture_root.join(path)
    }
}

fn resolve_manifest_relative_path(manifest_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        manifest_dir.join(path)
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
