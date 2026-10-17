//! Acquire local records and reuse canonical region and GC-bias summaries.
use super::model::{
    BamCoverageSampleTruth, BamCoverageTruthCase, BamGcBiasSampleTruth, BamGcBiasTruthCase,
    BamGcCoverageTruthBundle, BamGcCoverageTruthManifest,
};
use super::{BAM_GC_COVERAGE_TRUTH_BUNDLE_SCHEMA_VERSION, BAM_GC_COVERAGE_TRUTH_FIXTURE_ID};
use crate::bam::{
    load_bam_corpus_fixture_manifest_path, BamCorpusFixtureManifest, BamCorpusFixtureSample,
};
use crate::paths::{
    path_relative_to_repo, resolve_manifest_relative_path,
    resolve_manifest_relative_path as resolve_repo_relative_path,
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::{summarize_tiny_bam_coverage_truth, summarize_tiny_bam_gc_bias_truth};
use std::fs;
use std::path::Path;

pub(super) fn load_bam_gc_coverage_truth_manifest_path(
    manifest_path: &Path,
) -> Result<BamGcCoverageTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn load_bam_gc_coverage_truth_bundle(
    expected_path: &Path,
) -> Result<BamGcCoverageTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

pub(super) fn build_actual_truth_bundle(
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
