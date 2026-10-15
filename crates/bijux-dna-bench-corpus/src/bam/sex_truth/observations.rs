//! Acquire local observations and normalize the existing domain sex-summary wire.
use super::model::{
    BamSexSampleTruth, BamSexTruthBundle, BamSexTruthCase, BamSexTruthManifest, BamSexTruthSummary,
};
use super::{BAM_SEX_TRUTH_BUNDLE_SCHEMA_VERSION, BAM_SEX_TRUTH_FIXTURE_ID};
use crate::bam::{load_bam_corpus_fixture_manifest_path, BamCorpusFixtureSample};
use crate::paths::{
    path_relative_to_repo, resolve_manifest_relative_path,
    resolve_manifest_relative_path as resolve_repo_relative_path,
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::{summarize_tiny_bam_sex, BamSexSummaryV1};
use std::fs;
use std::path::Path;

pub(super) fn load_bam_sex_truth_manifest_path(
    manifest_path: &Path,
) -> Result<BamSexTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn load_bam_sex_truth_bundle(expected_path: &Path) -> Result<BamSexTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

pub(super) fn build_actual_truth_bundle(
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
