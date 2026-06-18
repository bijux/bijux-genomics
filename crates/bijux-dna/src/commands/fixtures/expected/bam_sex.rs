#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::metrics::SexConfidenceClass;
use bijux_dna_domain_bam::{
    summarize_tiny_bam_sex, BamSexSummaryV1, BAM_SEX_SUMMARY_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_corpus_fixture::bam::{
    validate_bam_corpus_fixture_manifest_path, BamCorpusFixtureManifest, BamCorpusFixtureSample,
};

pub(crate) const BAM_SEX_TRUTH_FIXTURE_ID: &str = "sex-inference-truth";
pub(crate) const BAM_SEX_TRUTH_MANIFEST_SCHEMA_VERSION: &str = "bijux.bench.bam_sex_truth.v1";
const BAM_SEX_TRUTH_BUNDLE_SCHEMA_VERSION: &str = "bijux.bench.bam_sex_truth.expected.v1";
const BAM_SEX_TRUTH_VALIDATION_SCHEMA_VERSION: &str = "bijux.bench.bam_sex_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BamSexTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    corpus_manifest_path: PathBuf,
    expected_path: PathBuf,
    source_paths: Vec<PathBuf>,
    cases: Vec<BamSexTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BamSexTruthCase {
    sample_id: String,
    method: String,
    chromosome_system: String,
    minimum_y_sites: u32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamSexTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_case_count: usize,
    pub(crate) validated_ok_case_count: usize,
    pub(crate) validated_insufficient_case_count: usize,
    pub(crate) validated_call_classes: Vec<String>,
    pub(crate) checked_samples: Vec<String>,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BamSexTruthBundle {
    schema_version: String,
    fixture_id: String,
    sample_truths: Vec<BamSexSampleTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BamSexSampleTruth {
    sample_id: String,
    cohort: String,
    alignment_path: String,
    reference_path: String,
    summary: BamSexTruthSummary,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BamSexTruthSummary {
    summary_schema_version: String,
    stage_id: String,
    method: String,
    #[serde(default)]
    chromosome_system: Option<String>,
    #[serde(default)]
    minimum_y_sites: Option<u32>,
    x_contig: String,
    y_contig: String,
    autosomal_contigs: Vec<String>,
    x_coverage: f64,
    y_coverage: f64,
    autosomal_coverage: f64,
    x_covered_sites: u64,
    y_covered_sites: u64,
    #[serde(default)]
    x_to_y_ratio: Option<f64>,
    call: SexConfidenceClass,
    confidence: f64,
    status: String,
    #[serde(default)]
    insufficiency_reason: Option<String>,
}

pub(crate) fn validate_bam_sex_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamSexTruthValidationReport> {
    let manifest = load_bam_sex_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("BAM sex truth manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("BAM sex truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_bam_sex_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    let expected_map = expected
        .sample_truths
        .iter()
        .map(|sample| (sample.sample_id.as_str(), sample))
        .collect::<BTreeMap<_, _>>();
    let actual_map = actual
        .sample_truths
        .iter()
        .map(|sample| (sample.sample_id.as_str(), sample))
        .collect::<BTreeMap<_, _>>();
    if expected_map.len() != actual_map.len() {
        return Err(anyhow!(
            "BAM sex truth case count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for case in &manifest.cases {
        let expected_sample = expected_map.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!("expected BAM sex truth is missing sample `{}`", case.sample_id)
        })?;
        let actual_sample = actual_map.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!("observed BAM sex truth is missing sample `{}`", case.sample_id)
        })?;
        if expected_sample != actual_sample {
            return Err(anyhow!(
                "BAM sex truth drifted for sample `{}`\nexpected: {expected_sample:#?}\nobserved: {actual_sample:#?}",
                case.sample_id
            ));
        }
    }

    let validated_call_classes = collect_call_classes(&actual.sample_truths);
    validate_required_call_classes(&validated_call_classes, manifest_path)?;
    let validated_ok_case_count =
        actual.sample_truths.iter().filter(|sample| sample.summary.status == "ok").count();
    let validated_insufficient_case_count = actual
        .sample_truths
        .iter()
        .filter(|sample| sample.summary.call == SexConfidenceClass::Insufficient)
        .count();

    Ok(BamSexTruthValidationReport {
        schema_version: BAM_SEX_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_case_count: actual.sample_truths.len(),
        validated_ok_case_count,
        validated_insufficient_case_count,
        validated_call_classes,
        checked_samples: manifest.cases.iter().map(|case| case.sample_id.clone()).collect(),
        valid: true,
    })
}

fn load_bam_sex_truth_manifest_path(manifest_path: &Path) -> Result<BamSexTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &BamSexTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != BAM_SEX_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM sex truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            BAM_SEX_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != BAM_SEX_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "BAM sex truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            BAM_SEX_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    if !corpus_manifest_path.is_file() {
        return Err(anyhow!(
            "BAM sex truth corpus manifest is missing: {}",
            corpus_manifest_path.display()
        ));
    }
    validate_bam_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "BAM sex truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "BAM sex truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "BAM sex truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }

    let mut sample_ids = BTreeSet::new();
    for case in &manifest.cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM sex truth manifest `{}` contains an empty sample_id",
                manifest_path.display()
            ));
        }
        if !sample_ids.insert(case.sample_id.clone()) {
            return Err(anyhow!("BAM sex truth manifest repeats sample_id `{}`", case.sample_id));
        }
        if case.method.trim().is_empty() {
            return Err(anyhow!(
                "BAM sex truth case `{}` must declare a non-empty method",
                case.sample_id
            ));
        }
        if case.chromosome_system.trim().is_empty() {
            return Err(anyhow!(
                "BAM sex truth case `{}` must declare a non-empty chromosome_system",
                case.sample_id
            ));
        }
    }

    Ok(())
}

fn load_bam_sex_truth_bundle(expected_path: &Path) -> Result<BamSexTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
    manifest: &BamSexTruthManifest,
    bundle: &BamSexTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != BAM_SEX_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM sex truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            BAM_SEX_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM sex truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let actual_sample_ids =
        bundle.sample_truths.iter().map(|sample| sample.sample_id.as_str()).collect::<Vec<_>>();
    let expected_sample_ids =
        manifest.cases.iter().map(|case| case.sample_id.as_str()).collect::<Vec<_>>();
    if actual_sample_ids != expected_sample_ids {
        return Err(anyhow!(
            "BAM sex truth bundle `{}` must contain samples {:?}",
            expected_path.display(),
            expected_sample_ids
        ));
    }
    for sample in &bundle.sample_truths {
        validate_sample_truth_contract(sample, expected_path)?;
    }
    validate_required_call_classes(&collect_call_classes(&bundle.sample_truths), expected_path)?;
    Ok(())
}

fn validate_sample_truth_contract(sample: &BamSexSampleTruth, expected_path: &Path) -> Result<()> {
    if sample.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth bundle `{}` contains a sample with an empty sample_id",
            expected_path.display()
        ));
    }
    if sample.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must declare a non-empty cohort",
            sample.sample_id
        ));
    }
    if sample.alignment_path.trim().is_empty() || sample.reference_path.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must declare non-empty input paths",
            sample.sample_id
        ));
    }
    if sample.summary.summary_schema_version != BAM_SEX_SUMMARY_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM sex truth sample `{}` uses schema `{}` instead of `{}`",
            sample.sample_id,
            sample.summary.summary_schema_version,
            BAM_SEX_SUMMARY_SCHEMA_VERSION
        ));
    }
    if sample.summary.stage_id != "bam.sex" {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must keep stage_id `bam.sex`",
            sample.sample_id
        ));
    }
    if sample.summary.method.trim().is_empty() || sample.summary.status.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must declare non-empty method and status",
            sample.sample_id
        ));
    }
    if sample.summary.x_contig.trim().is_empty() || sample.summary.y_contig.trim().is_empty() {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must declare non-empty X/Y contig names",
            sample.sample_id
        ));
    }
    if sample.summary.x_coverage < 0.0
        || sample.summary.y_coverage < 0.0
        || sample.summary.autosomal_coverage < 0.0
    {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must keep non-negative coverage values",
            sample.sample_id
        ));
    }
    if sample.summary.call == SexConfidenceClass::Insufficient {
        if sample.summary.insufficiency_reason.as_deref().unwrap_or_default().trim().is_empty() {
            return Err(anyhow!(
                "BAM sex truth sample `{}` must declare insufficiency_reason when call is insufficient",
                sample.sample_id
            ));
        }
        if sample.summary.confidence != 0.0 {
            return Err(anyhow!(
                "BAM sex truth sample `{}` must keep confidence zero when call is insufficient",
                sample.sample_id
            ));
        }
    } else if sample.summary.status == "ok" && sample.summary.insufficiency_reason.is_some() {
        return Err(anyhow!(
            "BAM sex truth sample `{}` must not declare insufficiency_reason when status is ok",
            sample.sample_id
        ));
    }
    Ok(())
}

fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &BamSexTruthManifest,
) -> Result<BamSexTruthBundle> {
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    let corpus = load_bam_corpus_fixture_manifest_path(&corpus_manifest_path)?;
    let manifest_dir = corpus_manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM corpus fixture manifest has no parent directory: {}",
            corpus_manifest_path.display()
        )
    })?;
    let reference_path = resolve_manifest_relative_path(manifest_dir, &corpus.reference_fasta);

    let sample_truths = manifest
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
            build_sample_truth(repo_root, manifest_dir, sample, &reference_path, case)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BamSexTruthBundle {
        schema_version: BAM_SEX_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: BAM_SEX_TRUTH_FIXTURE_ID.to_string(),
        sample_truths,
    })
}

fn load_bam_corpus_fixture_manifest_path(manifest_path: &Path) -> Result<BamCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn build_sample_truth(
    repo_root: &Path,
    manifest_dir: &Path,
    sample: &BamCorpusFixtureSample,
    reference_path: &Path,
    case: &BamSexTruthCase,
) -> Result<BamSexSampleTruth> {
    let alignment_path = resolve_manifest_relative_path(manifest_dir, &sample.alignment_path);
    let summary = summarize_tiny_bam_sex(
        &alignment_path,
        reference_path,
        &case.method,
        Some(case.chromosome_system.as_str()),
        Some(case.minimum_y_sites),
    )?;
    Ok(BamSexSampleTruth {
        sample_id: sample.sample_id.clone(),
        cohort: sample.cohort.clone(),
        alignment_path: path_relative_to_repo(repo_root, &alignment_path),
        reference_path: path_relative_to_repo(repo_root, reference_path),
        summary: normalize_sex_summary(summary),
    })
}

fn normalize_sex_summary(summary: BamSexSummaryV1) -> BamSexTruthSummary {
    BamSexTruthSummary {
        summary_schema_version: summary.schema_version,
        stage_id: summary.stage_id,
        method: summary.method,
        chromosome_system: summary.chromosome_system,
        minimum_y_sites: summary.minimum_y_sites,
        x_contig: summary.x_contig,
        y_contig: summary.y_contig,
        autosomal_contigs: summary.autosomal_contigs,
        x_coverage: summary.x_coverage,
        y_coverage: summary.y_coverage,
        autosomal_coverage: summary.autosomal_coverage,
        x_covered_sites: summary.x_covered_sites,
        y_covered_sites: summary.y_covered_sites,
        x_to_y_ratio: summary.x_to_y_ratio,
        call: summary.call,
        confidence: summary.confidence,
        status: summary.status,
        insufficiency_reason: summary.insufficiency_reason,
    }
}

fn validate_required_call_classes(call_classes: &[String], path: &Path) -> Result<()> {
    let required = ["male", "female", "ambiguous", "insufficient"];
    for call_class in required {
        if !call_classes.iter().any(|value| value == call_class) {
            return Err(anyhow!(
                "BAM sex truth `{}` must cover the `{call_class}` call class",
                path.display()
            ));
        }
    }
    Ok(())
}

fn collect_call_classes(sample_truths: &[BamSexSampleTruth]) -> Vec<String> {
    let mut classes = sample_truths
        .iter()
        .map(|sample| sex_call_name(sample.summary.call).to_string())
        .collect::<Vec<_>>();
    classes.sort();
    classes.dedup();
    classes
}

fn sex_call_name(call: SexConfidenceClass) -> &'static str {
    match call {
        SexConfidenceClass::Male => "male",
        SexConfidenceClass::Female => "female",
        SexConfidenceClass::Ambiguous => "ambiguous",
        SexConfidenceClass::Insufficient => "insufficient",
    }
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
