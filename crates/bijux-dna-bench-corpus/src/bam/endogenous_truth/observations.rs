//! Acquire local declarations and reuse endogenous domain summaries.
use super::model::{
    BamEndogenousTruthBundle, BamEndogenousTruthCase, BamEndogenousTruthManifest,
    BamEndogenousTruthSample,
};
use super::{BAM_ENDOGENOUS_TRUTH_BUNDLE_SCHEMA_VERSION, BAM_ENDOGENOUS_TRUTH_FIXTURE_ID};
use crate::bam::{load_bam_corpus_fixture_manifest_path, BamCorpusFixtureSample};
use crate::paths::{
    path_relative_to_repo, resolve_manifest_relative_path,
    resolve_manifest_relative_path as resolve_repo_relative_path,
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::summarize_tiny_bam_endogenous_truth;
use std::fs;
use std::path::Path;

pub(super) fn load_bam_endogenous_truth_manifest_path(
    manifest_path: &Path,
) -> Result<BamEndogenousTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn load_bam_endogenous_truth_bundle(
    expected_path: &Path,
) -> Result<BamEndogenousTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

pub(super) fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &BamEndogenousTruthManifest,
) -> Result<BamEndogenousTruthBundle> {
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    let corpus = load_bam_corpus_fixture_manifest_path(&corpus_manifest_path)?;
    let manifest_dir = corpus_manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM corpus fixture manifest has no parent directory: {}",
            corpus_manifest_path.display()
        )
    })?;

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
            build_sample_truth(repo_root, manifest_dir, sample, case)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BamEndogenousTruthBundle {
        schema_version: BAM_ENDOGENOUS_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: BAM_ENDOGENOUS_TRUTH_FIXTURE_ID.to_string(),
        sample_truths,
    })
}

fn build_sample_truth(
    repo_root: &Path,
    manifest_dir: &Path,
    sample: &BamCorpusFixtureSample,
    case: &BamEndogenousTruthCase,
) -> Result<BamEndogenousTruthSample> {
    let alignment_path = resolve_manifest_relative_path(manifest_dir, &sample.alignment_path);
    let summary = summarize_tiny_bam_endogenous_truth(
        &alignment_path,
        &case.method,
        &case.host_reference_scope,
        Some(case.prealignment_fraction),
    )?;
    Ok(BamEndogenousTruthSample {
        sample_id: sample.sample_id.clone(),
        cohort: sample.cohort.clone(),
        alignment_path: path_relative_to_repo(repo_root, &alignment_path),
        summary,
    })
}
