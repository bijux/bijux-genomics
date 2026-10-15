//! Acquire local case observations and normalize the existing domain summary wire.
use super::model::{
    BamHaplogroupCaseTruth, BamHaplogroupTruthBundle, BamHaplogroupTruthCase,
    BamHaplogroupTruthManifest, BamHaplogroupTruthSummary,
};
use super::{BAM_HAPLOGROUP_TRUTH_BUNDLE_SCHEMA_VERSION, BAM_HAPLOGROUP_TRUTH_FIXTURE_ID};
use crate::bam::{load_bam_corpus_fixture_manifest_path, BamCorpusFixtureSample};
use crate::paths::{
    path_relative_to_repo, resolve_manifest_relative_path,
    resolve_manifest_relative_path as resolve_repo_relative_path,
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::{summarize_tiny_bam_haplogroup_truth, BamHaplogroupTruthSummaryV1};
use std::fs;
use std::path::Path;

pub(super) fn load_bam_haplogroup_truth_manifest_path(
    manifest_path: &Path,
) -> Result<BamHaplogroupTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

pub(super) fn load_bam_haplogroup_truth_bundle(
    expected_path: &Path,
) -> Result<BamHaplogroupTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

pub(super) fn build_actual_truth_bundle(
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
