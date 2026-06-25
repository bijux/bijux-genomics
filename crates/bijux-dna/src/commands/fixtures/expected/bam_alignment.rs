#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::{
    summarize_tiny_bam_alignment_truth, BamAlignmentTruthCigarClassV1,
    BamAlignmentTruthMapqClassV1, BamAlignmentTruthPositionV1, BamAlignmentTruthRecordV1,
};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_corpus_fixture::bam::{
    validate_bam_corpus_fixture_manifest_path, BamCorpusFixtureManifest, BamCorpusFixtureSample,
};

pub(crate) const BAM_ALIGNMENT_TRUTH_FIXTURE_ID: &str = "bam-alignment-truth";
pub(crate) const BAM_ALIGNMENT_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.bam_alignment_truth.v1";
const BAM_ALIGNMENT_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.bam_alignment_truth.expected.v1";
const BAM_ALIGNMENT_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_alignment_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BamAlignmentTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    corpus_manifest_path: PathBuf,
    expected_path: PathBuf,
    source_paths: Vec<PathBuf>,
    sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamAlignmentTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_sample_count: usize,
    pub(crate) validated_record_count: usize,
    pub(crate) valid: bool,
    pub(crate) checked_samples: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BamAlignmentTruthBundle {
    schema_version: String,
    fixture_id: String,
    sample_truths: Vec<BamAlignmentSampleTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct BamAlignmentSampleTruth {
    sample_id: String,
    cohort: String,
    alignment_path: String,
    total_reads: u64,
    mapped_reads: u64,
    unmapped_reads: u64,
    mapped_contigs: Vec<String>,
    mapped_positions: Vec<BamAlignmentTruthPositionV1>,
    mapped_mapq_classes: Vec<BamAlignmentTruthMapqClassV1>,
    cigar_classes: Vec<BamAlignmentTruthCigarClassV1>,
    records: Vec<BamAlignmentTruthRecordV1>,
}

pub(crate) fn validate_bam_alignment_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamAlignmentTruthValidationReport> {
    let manifest = load_bam_alignment_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!("BAM alignment truth manifest has no parent directory: {}", manifest_path.display())
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("BAM alignment truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_bam_alignment_truth_bundle(&expected_path)?;
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
            "BAM alignment truth sample count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for sample_id in &manifest.sample_ids {
        let expected_sample = expected_map.get(sample_id.as_str()).ok_or_else(|| {
            anyhow!("expected BAM alignment truth is missing sample `{sample_id}`")
        })?;
        let actual_sample = actual_map.get(sample_id.as_str()).ok_or_else(|| {
            anyhow!("observed BAM alignment truth is missing sample `{sample_id}`")
        })?;
        if expected_sample != actual_sample {
            return Err(anyhow!("BAM alignment truth drifted for sample `{sample_id}`"));
        }
    }

    let checked_samples =
        actual.sample_truths.iter().map(|sample| sample.sample_id.clone()).collect::<Vec<_>>();
    let validated_record_count =
        actual.sample_truths.iter().map(|sample| sample.records.len()).sum::<usize>();

    Ok(BamAlignmentTruthValidationReport {
        schema_version: BAM_ALIGNMENT_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_sample_count: checked_samples.len(),
        validated_record_count,
        valid: true,
        checked_samples,
    })
}

fn load_bam_alignment_truth_manifest_path(
    manifest_path: &Path,
) -> Result<BamAlignmentTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &BamAlignmentTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != BAM_ALIGNMENT_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM alignment truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            BAM_ALIGNMENT_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != BAM_ALIGNMENT_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "BAM alignment truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            BAM_ALIGNMENT_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "BAM alignment truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    if !corpus_manifest_path.is_file() {
        return Err(anyhow!(
            "BAM alignment truth corpus manifest is missing: {}",
            corpus_manifest_path.display()
        ));
    }
    validate_bam_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "BAM alignment truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "BAM alignment truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.sample_ids.is_empty() {
        return Err(anyhow!(
            "BAM alignment truth manifest `{}` must declare at least one sample_id",
            manifest_path.display()
        ));
    }
    let mut sample_ids = BTreeSet::new();
    for sample_id in &manifest.sample_ids {
        if sample_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM alignment truth manifest `{}` contains an empty sample_id",
                manifest_path.display()
            ));
        }
        if !sample_ids.insert(sample_id.clone()) {
            return Err(anyhow!("BAM alignment truth manifest repeats sample_id `{sample_id}`"));
        }
    }
    Ok(())
}

fn load_bam_alignment_truth_bundle(expected_path: &Path) -> Result<BamAlignmentTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
    manifest: &BamAlignmentTruthManifest,
    bundle: &BamAlignmentTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != BAM_ALIGNMENT_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM alignment truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            BAM_ALIGNMENT_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM alignment truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
            bundle.fixture_id,
            manifest.fixture_id
        ));
    }
    let actual_sample_ids =
        bundle.sample_truths.iter().map(|sample| sample.sample_id.as_str()).collect::<Vec<_>>();
    let expected_sample_ids = manifest.sample_ids.iter().map(String::as_str).collect::<Vec<_>>();
    if actual_sample_ids != expected_sample_ids {
        return Err(anyhow!(
            "BAM alignment truth bundle `{}` must contain samples {:?}",
            expected_path.display(),
            expected_sample_ids
        ));
    }
    for sample in &bundle.sample_truths {
        validate_sample_truth_contract(sample, expected_path)?;
    }
    Ok(())
}

fn validate_sample_truth_contract(
    sample: &BamAlignmentSampleTruth,
    expected_path: &Path,
) -> Result<()> {
    if sample.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM alignment truth bundle `{}` contains a sample with an empty sample_id",
            expected_path.display()
        ));
    }
    if sample.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` must declare a non-empty cohort",
            sample.sample_id
        ));
    }
    if sample.alignment_path.trim().is_empty() {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` must declare a non-empty alignment_path",
            sample.sample_id
        ));
    }
    if sample.total_reads != sample.records.len() as u64 {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` total_reads {} does not equal record count {}",
            sample.sample_id,
            sample.total_reads,
            sample.records.len()
        ));
    }
    if sample.total_reads != sample.mapped_reads + sample.unmapped_reads {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` total_reads {} does not equal mapped+unmapped {}",
            sample.sample_id,
            sample.total_reads,
            sample.mapped_reads + sample.unmapped_reads
        ));
    }
    let expected_mapped_contigs = sample
        .records
        .iter()
        .filter(|record| record.mapped)
        .filter_map(|record| record.reference_name.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    if expected_mapped_contigs != sample.mapped_contigs {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` mapped_contigs drifted",
            sample.sample_id
        ));
    }
    let mapped_position_total =
        sample.mapped_positions.iter().map(|row| row.read_count).sum::<u64>();
    if mapped_position_total != sample.mapped_reads {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` mapped_positions count {} does not equal mapped_reads {}",
            sample.sample_id,
            mapped_position_total,
            sample.mapped_reads
        ));
    }
    let mapped_mapq_total =
        sample.mapped_mapq_classes.iter().map(|row| row.read_count).sum::<u64>();
    if mapped_mapq_total != sample.mapped_reads {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` mapped MAPQ class count {} does not equal mapped_reads {}",
            sample.sample_id,
            mapped_mapq_total,
            sample.mapped_reads
        ));
    }
    let cigar_total = sample.cigar_classes.iter().map(|row| row.read_count).sum::<u64>();
    if cigar_total != sample.total_reads {
        return Err(anyhow!(
            "BAM alignment truth sample `{}` cigar class count {} does not equal total_reads {}",
            sample.sample_id,
            cigar_total,
            sample.total_reads
        ));
    }
    Ok(())
}

fn build_actual_truth_bundle(
    repo_root: &Path,
    manifest: &BamAlignmentTruthManifest,
) -> Result<BamAlignmentTruthBundle> {
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

fn load_bam_corpus_fixture_manifest_path(manifest_path: &Path) -> Result<BamCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
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
