//! Read declared manifests and ask the BAM domain for canonical alignment observations.
use super::{
    path_relative_to_repo, resolve_manifest_relative_path, BamAlignmentSampleTruth,
    BamAlignmentTruthBundle, BamAlignmentTruthManifest, BAM_ALIGNMENT_TRUTH_BUNDLE_SCHEMA_VERSION,
    BAM_ALIGNMENT_TRUTH_FIXTURE_ID,
};
use crate::bam::{load_bam_corpus_fixture_manifest_path, BamCorpusFixtureSample};
use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::summarize_tiny_bam_alignment_truth;
use std::fs;
use std::path::Path;

pub(super) fn load_bam_alignment_truth_manifest_path(
    manifest_path: &Path,
) -> Result<BamAlignmentTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn load_bam_alignment_truth_bundle(
    expected_path: &Path,
) -> Result<BamAlignmentTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

pub(super) fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &BamAlignmentTruthManifest,
) -> Result<BamAlignmentTruthBundle> {
    let corpus_manifest_path =
        resolve_manifest_relative_path(repo_root, &manifest.corpus_manifest_path);
    let corpus = load_bam_corpus_fixture_manifest_path(&corpus_manifest_path)?;
    let manifest_dir = corpus_manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM corpus fixture manifest has no parent directory: {}",
            corpus_manifest_path.display()
        )
    })?;

    let sample_truths = manifest
        .sample_ids
        .iter()
        .map(|sample_id| {
            let sample = corpus
                .samples
                .iter()
                .find(|sample| sample.sample_id == *sample_id)
                .ok_or_else(|| anyhow!("BAM corpus fixture is missing sample `{sample_id}`"))?;
            build_sample_truth(repo_root, manifest_dir, sample)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(BamAlignmentTruthBundle {
        schema_version: BAM_ALIGNMENT_TRUTH_BUNDLE_SCHEMA_VERSION.to_string(),
        fixture_id: BAM_ALIGNMENT_TRUTH_FIXTURE_ID.to_string(),
        sample_truths,
    })
}

fn build_sample_truth(
    repo_root: &Path,
    manifest_dir: &Path,
    sample: &BamCorpusFixtureSample,
) -> Result<BamAlignmentSampleTruth> {
    let alignment_path = resolve_manifest_relative_path(manifest_dir, &sample.alignment_path);
    let summary = summarize_tiny_bam_alignment_truth(&alignment_path)?;
    Ok(BamAlignmentSampleTruth {
        sample_id: sample.sample_id.clone(),
        cohort: sample.cohort.clone(),
        alignment_path: path_relative_to_repo(repo_root, &alignment_path),
        total_reads: summary.total_reads,
        mapped_reads: summary.mapped_reads,
        unmapped_reads: summary.unmapped_reads,
        mapped_contigs: summary.mapped_contigs,
        mapped_positions: summary.mapped_positions,
        mapped_mapq_classes: summary.mapped_mapq_classes,
        cigar_classes: summary.cigar_classes,
        records: summary.records,
    })
}
