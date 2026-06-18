#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::{
    summarize_tiny_bam_coverage_truth, summarize_tiny_bam_gc_bias_truth, BamCoverageTruthSummaryV1,
    BamGcBiasTruthSummaryV1, BAM_COVERAGE_TRUTH_SUMMARY_SCHEMA_VERSION,
    BAM_GC_BIAS_TRUTH_SUMMARY_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_corpus_fixture::bam::{
    validate_bam_corpus_fixture_manifest_path, BamCorpusFixtureManifest, BamCorpusFixtureSample,
};

pub(crate) const BAM_GC_COVERAGE_TRUTH_FIXTURE_ID: &str = "bam-gc-coverage-truth";
pub(crate) const BAM_GC_COVERAGE_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.bam_gc_coverage_truth.v1";
const BAM_GC_COVERAGE_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.bam_gc_coverage_truth.expected.v1";
const BAM_GC_COVERAGE_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_gc_coverage_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BamGcCoverageTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    corpus_manifest_path: PathBuf,
    expected_path: PathBuf,
    source_paths: Vec<PathBuf>,
    coverage_cases: Vec<BamCoverageTruthCase>,
    gc_bias_cases: Vec<BamGcBiasTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BamCoverageTruthCase {
    sample_id: String,
    regions_path: PathBuf,
    depth_thresholds: Vec<u32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BamGcBiasTruthCase {
    sample_id: String,
    reference_path: PathBuf,
    window_size: u32,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamGcCoverageTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_coverage_sample_count: usize,
    pub(crate) validated_gc_bias_sample_count: usize,
    pub(crate) validated_coverage_region_count: usize,
    pub(crate) validated_gc_bias_bin_count: usize,
    pub(crate) valid: bool,
    pub(crate) checked_samples: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BamGcCoverageTruthBundle {
    schema_version: String,
    fixture_id: String,
    coverage_truths: Vec<BamCoverageSampleTruth>,
    gc_bias_truths: Vec<BamGcBiasSampleTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BamCoverageSampleTruth {
    sample_id: String,
    cohort: String,
    alignment_path: String,
    regions_path: String,
    summary: BamCoverageTruthSummaryV1,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BamGcBiasSampleTruth {
    sample_id: String,
    cohort: String,
    alignment_path: String,
    reference_path: String,
    summary: BamGcBiasTruthSummaryV1,
}

pub(crate) fn validate_bam_gc_coverage_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamGcCoverageTruthValidationReport> {
    let manifest = load_bam_gc_coverage_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM GC-bias/coverage truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle is missing: {}",
            expected_path.display()
        ));
    }

    let expected = load_bam_gc_coverage_truth_bundle(&expected_path)?;
    validate_bundle_contract(&manifest, &expected, &expected_path)?;

    let actual = build_actual_truth_bundle(repo_root, &manifest)?;
    let expected_coverage = expected
        .coverage_truths
        .iter()
        .map(|sample| (sample.sample_id.as_str(), sample))
        .collect::<BTreeMap<_, _>>();
    let actual_coverage = actual
        .coverage_truths
        .iter()
        .map(|sample| (sample.sample_id.as_str(), sample))
        .collect::<BTreeMap<_, _>>();
    if expected_coverage.len() != actual_coverage.len() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample count drifted: expected {}, observed {}",
            expected_coverage.len(),
            actual_coverage.len()
        ));
    }
    for case in &manifest.coverage_cases {
        let expected_sample = expected_coverage.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!(
                "expected BAM GC-bias/coverage truth is missing coverage sample `{}`",
                case.sample_id
            )
        })?;
        let actual_sample = actual_coverage.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!(
                "observed BAM GC-bias/coverage truth is missing coverage sample `{}`",
                case.sample_id
            )
        })?;
        if expected_sample != actual_sample {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth coverage sample drifted for `{}`",
                case.sample_id
            ));
        }
    }

    let expected_gc_bias = expected
        .gc_bias_truths
        .iter()
        .map(|sample| (sample.sample_id.as_str(), sample))
        .collect::<BTreeMap<_, _>>();
    let actual_gc_bias = actual
        .gc_bias_truths
        .iter()
        .map(|sample| (sample.sample_id.as_str(), sample))
        .collect::<BTreeMap<_, _>>();
    if expected_gc_bias.len() != actual_gc_bias.len() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample count drifted: expected {}, observed {}",
            expected_gc_bias.len(),
            actual_gc_bias.len()
        ));
    }
    for case in &manifest.gc_bias_cases {
        let expected_sample = expected_gc_bias.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!(
                "expected BAM GC-bias/coverage truth is missing GC-bias sample `{}`",
                case.sample_id
            )
        })?;
        let actual_sample = actual_gc_bias.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!(
                "observed BAM GC-bias/coverage truth is missing GC-bias sample `{}`",
                case.sample_id
            )
        })?;
        if expected_sample != actual_sample {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth GC-bias sample drifted for `{}`",
                case.sample_id
            ));
        }
    }

    let checked_samples = manifest
        .coverage_cases
        .iter()
        .map(|case| case.sample_id.clone())
        .chain(manifest.gc_bias_cases.iter().map(|case| case.sample_id.clone()))
        .collect::<Vec<_>>();
    let validated_coverage_region_count =
        actual.coverage_truths.iter().map(|sample| sample.summary.region_summaries.len()).sum();
    let validated_gc_bias_bin_count =
        actual.gc_bias_truths.iter().map(|sample| sample.summary.gc_bins.len()).sum();

    Ok(BamGcCoverageTruthValidationReport {
        schema_version: BAM_GC_COVERAGE_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_coverage_sample_count: actual.coverage_truths.len(),
        validated_gc_bias_sample_count: actual.gc_bias_truths.len(),
        validated_coverage_region_count,
        validated_gc_bias_bin_count,
        valid: true,
        checked_samples,
    })
}

fn load_bam_gc_coverage_truth_manifest_path(
    manifest_path: &Path,
) -> Result<BamGcCoverageTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &BamGcCoverageTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != BAM_GC_COVERAGE_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            BAM_GC_COVERAGE_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != BAM_GC_COVERAGE_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            BAM_GC_COVERAGE_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }

    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    if !corpus_manifest_path.is_file() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth corpus manifest is missing: {}",
            corpus_manifest_path.display()
        ));
    }
    validate_bam_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;

    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }

    if manifest.coverage_cases.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` must declare at least one coverage case",
            manifest_path.display()
        ));
    }
    if manifest.gc_bias_cases.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth manifest `{}` must declare at least one GC-bias case",
            manifest_path.display()
        ));
    }

    let mut coverage_sample_ids = BTreeSet::new();
    for case in &manifest.coverage_cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth manifest `{}` contains an empty coverage sample_id",
                manifest_path.display()
            ));
        }
        if !coverage_sample_ids.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth manifest repeats coverage sample_id `{}`",
                case.sample_id
            ));
        }
        if case.depth_thresholds.is_empty() {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth coverage sample `{}` must declare depth thresholds",
                case.sample_id
            ));
        }
        let regions_path = resolve_repo_relative_path(repo_root, &case.regions_path);
        if !regions_path.is_file() {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth coverage regions path is missing: {}",
                regions_path.display()
            ));
        }
    }

    let mut gc_bias_sample_ids = BTreeSet::new();
    for case in &manifest.gc_bias_cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth manifest `{}` contains an empty GC-bias sample_id",
                manifest_path.display()
            ));
        }
        if !gc_bias_sample_ids.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth manifest repeats GC-bias sample_id `{}`",
                case.sample_id
            ));
        }
        if case.window_size == 0 {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth GC-bias sample `{}` must declare window_size greater than zero",
                case.sample_id
            ));
        }
        let reference_path = resolve_repo_relative_path(repo_root, &case.reference_path);
        if !reference_path.is_file() {
            return Err(anyhow!(
                "BAM GC-bias/coverage truth reference path is missing: {}",
                reference_path.display()
            ));
        }
    }

    Ok(())
}

fn load_bam_gc_coverage_truth_bundle(expected_path: &Path) -> Result<BamGcCoverageTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
    manifest: &BamGcCoverageTruthManifest,
    bundle: &BamGcCoverageTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != BAM_GC_COVERAGE_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            BAM_GC_COVERAGE_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }

    let actual_coverage_ids =
        bundle.coverage_truths.iter().map(|sample| sample.sample_id.as_str()).collect::<Vec<_>>();
    let expected_coverage_ids =
        manifest.coverage_cases.iter().map(|case| case.sample_id.as_str()).collect::<Vec<_>>();
    if actual_coverage_ids != expected_coverage_ids {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle `{}` must contain coverage samples {:?}",
            expected_path.display(),
            expected_coverage_ids
        ));
    }

    let actual_gc_bias_ids =
        bundle.gc_bias_truths.iter().map(|sample| sample.sample_id.as_str()).collect::<Vec<_>>();
    let expected_gc_bias_ids =
        manifest.gc_bias_cases.iter().map(|case| case.sample_id.as_str()).collect::<Vec<_>>();
    if actual_gc_bias_ids != expected_gc_bias_ids {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle `{}` must contain GC-bias samples {:?}",
            expected_path.display(),
            expected_gc_bias_ids
        ));
    }

    for sample in &bundle.coverage_truths {
        validate_coverage_sample_truth_contract(sample, expected_path)?;
    }
    for sample in &bundle.gc_bias_truths {
        validate_gc_bias_sample_truth_contract(sample, expected_path)?;
    }
    Ok(())
}

fn validate_coverage_sample_truth_contract(
    sample: &BamCoverageSampleTruth,
    expected_path: &Path,
) -> Result<()> {
    if sample.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle `{}` contains a coverage sample with an empty sample_id",
            expected_path.display()
        ));
    }
    if sample.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` must declare a non-empty cohort",
            sample.sample_id
        ));
    }
    if sample.alignment_path.trim().is_empty() || sample.regions_path.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` must declare non-empty input paths",
            sample.sample_id
        ));
    }
    if sample.summary.schema_version != BAM_COVERAGE_TRUTH_SUMMARY_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` uses schema `{}` instead of `{}`",
            sample.sample_id,
            sample.summary.schema_version,
            BAM_COVERAGE_TRUTH_SUMMARY_SCHEMA_VERSION
        ));
    }
    if sample.summary.depth_thresholds.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` must declare depth thresholds",
            sample.sample_id
        ));
    }
    if sample.summary.region_summaries.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` must declare region summaries",
            sample.sample_id
        ));
    }
    if sample.summary.coverage_regime.trim().is_empty()
        || sample.summary.coverage_family.trim().is_empty()
    {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` must declare non-empty coverage labels",
            sample.sample_id
        ));
    }

    let total_bases = sample.summary.region_summaries.iter().map(|row| row.length).sum::<u64>();
    let covered_bases =
        sample.summary.region_summaries.iter().map(|row| row.covered_bases).sum::<u64>();
    if sample.summary.total_bases != total_bases {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` total_bases {} does not match region total {}",
            sample.sample_id,
            sample.summary.total_bases,
            total_bases
        ));
    }
    if sample.summary.covered_bases != covered_bases {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` covered_bases {} does not match region total {}",
            sample.sample_id,
            sample.summary.covered_bases,
            covered_bases
        ));
    }

    let expected_breadth =
        if total_bases == 0 { 0.0 } else { covered_bases as f64 / total_bases as f64 };
    if !approx_eq(sample.summary.breadth_1x, expected_breadth) {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` breadth_1x {} does not match region breadth {}",
            sample.sample_id,
            sample.summary.breadth_1x,
            expected_breadth
        ));
    }

    let weighted_depth_sum = sample
        .summary
        .region_summaries
        .iter()
        .map(|row| row.mean_depth * row.length as f64)
        .sum::<f64>();
    let expected_mean_depth =
        if total_bases == 0 { 0.0 } else { weighted_depth_sum / total_bases as f64 };
    if !approx_eq(sample.summary.mean_depth, expected_mean_depth) {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth coverage sample `{}` mean_depth {} does not match weighted region mean {}",
            sample.sample_id,
            sample.summary.mean_depth,
            expected_mean_depth
        ));
    }

    Ok(())
}

fn validate_gc_bias_sample_truth_contract(
    sample: &BamGcBiasSampleTruth,
    expected_path: &Path,
) -> Result<()> {
    if sample.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth bundle `{}` contains a GC-bias sample with an empty sample_id",
            expected_path.display()
        ));
    }
    if sample.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` must declare a non-empty cohort",
            sample.sample_id
        ));
    }
    if sample.alignment_path.trim().is_empty() || sample.reference_path.trim().is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` must declare non-empty input paths",
            sample.sample_id
        ));
    }
    if sample.summary.schema_version != BAM_GC_BIAS_TRUTH_SUMMARY_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` uses schema `{}` instead of `{}`",
            sample.sample_id,
            sample.summary.schema_version,
            BAM_GC_BIAS_TRUTH_SUMMARY_SCHEMA_VERSION
        ));
    }
    if sample.summary.window_size == 0 {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` must declare window_size greater than zero",
            sample.sample_id
        ));
    }
    if sample.summary.gc_bins.is_empty() {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` must declare GC bins",
            sample.sample_id
        ));
    }

    let windows = sample.summary.gc_bins.iter().map(|row| row.windows).sum::<u64>();
    let read_starts = sample.summary.gc_bins.iter().map(|row| row.read_starts).sum::<u64>();
    if sample.summary.windows != windows {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` windows {} does not match GC-bin total {}",
            sample.sample_id,
            sample.summary.windows,
            windows
        ));
    }
    if sample.summary.read_starts != read_starts {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` read_starts {} does not match GC-bin total {}",
            sample.sample_id,
            sample.summary.read_starts,
            read_starts
        ));
    }

    let expected_gc_bias_score = sample.summary.at_dropout.max(sample.summary.gc_dropout) / 100.0;
    if !approx_eq(sample.summary.gc_bias_score, expected_gc_bias_score) {
        return Err(anyhow!(
            "BAM GC-bias/coverage truth GC-bias sample `{}` gc_bias_score {} does not match dropout-derived score {}",
            sample.sample_id,
            sample.summary.gc_bias_score,
            expected_gc_bias_score
        ));
    }

    Ok(())
}

fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &BamGcCoverageTruthManifest,
) -> Result<BamGcCoverageTruthBundle> {
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    let corpus = load_bam_corpus_fixture_manifest_path(&corpus_manifest_path)?;
    let manifest_dir = corpus_manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM corpus fixture manifest has no parent directory: {}",
            corpus_manifest_path.display()
        )
    })?;

    let coverage_truths = manifest
        .coverage_cases
        .iter()
        .map(|case| build_coverage_sample_truth(repo_root, manifest_dir, &corpus, case))
        .collect::<Result<Vec<_>>>()?;
    let gc_bias_truths = manifest
        .gc_bias_cases
        .iter()
        .map(|case| build_gc_bias_sample_truth(repo_root, manifest_dir, &corpus, case))
        .collect::<Result<Vec<_>>>()?;

    Ok(BamGcCoverageTruthBundle {
        schema_version: BAM_GC_COVERAGE_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: BAM_GC_COVERAGE_TRUTH_FIXTURE_ID.to_string(),
        coverage_truths,
        gc_bias_truths,
    })
}

fn load_bam_corpus_fixture_manifest_path(manifest_path: &Path) -> Result<BamCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn build_coverage_sample_truth(
    repo_root: &Path,
    manifest_dir: &Path,
    corpus: &BamCorpusFixtureManifest,
    case: &BamCoverageTruthCase,
) -> Result<BamCoverageSampleTruth> {
    let sample = find_corpus_sample(corpus, &case.sample_id)?;
    let alignment_path = resolve_manifest_relative_path(manifest_dir, &sample.alignment_path);
    let regions_path = resolve_repo_relative_path(repo_root, &case.regions_path);
    let summary =
        summarize_tiny_bam_coverage_truth(&alignment_path, &regions_path, &case.depth_thresholds)?;
    Ok(BamCoverageSampleTruth {
        sample_id: sample.sample_id.clone(),
        cohort: sample.cohort.clone(),
        alignment_path: path_relative_to_repo(repo_root, &alignment_path),
        regions_path: path_relative_to_repo(repo_root, &regions_path),
        summary,
    })
}

fn build_gc_bias_sample_truth(
    repo_root: &Path,
    manifest_dir: &Path,
    corpus: &BamCorpusFixtureManifest,
    case: &BamGcBiasTruthCase,
) -> Result<BamGcBiasSampleTruth> {
    let sample = find_corpus_sample(corpus, &case.sample_id)?;
    let alignment_path = resolve_manifest_relative_path(manifest_dir, &sample.alignment_path);
    let reference_path = resolve_repo_relative_path(repo_root, &case.reference_path);
    let summary =
        summarize_tiny_bam_gc_bias_truth(&alignment_path, &reference_path, case.window_size)?;
    Ok(BamGcBiasSampleTruth {
        sample_id: sample.sample_id.clone(),
        cohort: sample.cohort.clone(),
        alignment_path: path_relative_to_repo(repo_root, &alignment_path),
        reference_path: path_relative_to_repo(repo_root, &reference_path),
        summary,
    })
}

fn find_corpus_sample<'a>(
    corpus: &'a BamCorpusFixtureManifest,
    sample_id: &str,
) -> Result<&'a BamCorpusFixtureSample> {
    corpus
        .samples
        .iter()
        .find(|sample| sample.sample_id == sample_id)
        .ok_or_else(|| anyhow!("BAM corpus fixture is missing sample `{sample_id}`"))
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

fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() <= 1e-9
}
