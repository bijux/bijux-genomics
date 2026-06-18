#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_bam::{
    summarize_tiny_bam_endogenous_truth, BamEndogenousTruthSummaryV1,
    BAM_ENDOGENOUS_TRUTH_SUMMARY_SCHEMA_VERSION,
};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_corpus_fixture::bam::{
    validate_bam_corpus_fixture_manifest_path, BamCorpusFixtureManifest, BamCorpusFixtureSample,
};

pub(crate) const BAM_ENDOGENOUS_TRUTH_FIXTURE_ID: &str = "endogenous-truth";
pub(crate) const BAM_ENDOGENOUS_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.bam_endogenous_truth.v1";
const BAM_ENDOGENOUS_TRUTH_BUNDLE_SCHEMA_VERSION: &str =
    "bijux.bench.bam_endogenous_truth.expected.v1";
const BAM_ENDOGENOUS_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.bam_endogenous_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BamEndogenousTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    corpus_manifest_path: PathBuf,
    expected_path: PathBuf,
    source_paths: Vec<PathBuf>,
    cases: Vec<BamEndogenousTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct BamEndogenousTruthCase {
    sample_id: String,
    method: String,
    host_reference_scope: String,
    prealignment_fraction: f64,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct BamEndogenousTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_path: String,
    pub(crate) validated_sample_count: usize,
    pub(crate) validated_total_reads: u64,
    pub(crate) validated_contaminant_reads: u64,
    pub(crate) validated_retained_reads: u64,
    pub(crate) checked_samples: Vec<String>,
    pub(crate) valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BamEndogenousTruthBundle {
    schema_version: String,
    fixture_id: String,
    sample_truths: Vec<BamEndogenousTruthSample>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BamEndogenousTruthSample {
    sample_id: String,
    cohort: String,
    alignment_path: String,
    summary: BamEndogenousTruthSummaryV1,
}

pub(crate) fn validate_bam_endogenous_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<BamEndogenousTruthValidationReport> {
    let manifest = load_bam_endogenous_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "BAM endogenous truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_path = resolve_fixture_path(fixture_root, &manifest.expected_path);
    if !expected_path.is_file() {
        return Err(anyhow!("BAM endogenous truth bundle is missing: {}", expected_path.display()));
    }

    let expected = load_bam_endogenous_truth_bundle(&expected_path)?;
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
            "BAM endogenous truth sample count drifted: expected {}, observed {}",
            expected_map.len(),
            actual_map.len()
        ));
    }
    for case in &manifest.cases {
        let expected_sample = expected_map.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!("expected BAM endogenous truth is missing sample `{}`", case.sample_id)
        })?;
        let actual_sample = actual_map.get(case.sample_id.as_str()).ok_or_else(|| {
            anyhow!("observed BAM endogenous truth is missing sample `{}`", case.sample_id)
        })?;
        if expected_sample != actual_sample {
            return Err(anyhow!("BAM endogenous truth drifted for sample `{}`", case.sample_id));
        }
    }

    let checked_samples =
        actual.sample_truths.iter().map(|sample| sample.sample_id.clone()).collect::<Vec<_>>();
    let validated_total_reads =
        actual.sample_truths.iter().map(|sample| sample.summary.total_reads).sum::<u64>();
    let validated_contaminant_reads =
        actual.sample_truths.iter().map(|sample| sample.summary.contaminant_reads).sum::<u64>();
    let validated_retained_reads =
        actual.sample_truths.iter().map(|sample| sample.summary.retained_reads).sum::<u64>();

    Ok(BamEndogenousTruthValidationReport {
        schema_version: BAM_ENDOGENOUS_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_path: path_relative_to_repo(repo_root, &expected_path),
        validated_sample_count: checked_samples.len(),
        validated_total_reads,
        validated_contaminant_reads,
        validated_retained_reads,
        checked_samples,
        valid: true,
    })
}

fn load_bam_endogenous_truth_manifest_path(
    manifest_path: &Path,
) -> Result<BamEndogenousTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &BamEndogenousTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != BAM_ENDOGENOUS_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM endogenous truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            BAM_ENDOGENOUS_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != BAM_ENDOGENOUS_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "BAM endogenous truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            BAM_ENDOGENOUS_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    if !corpus_manifest_path.is_file() {
        return Err(anyhow!(
            "BAM endogenous truth corpus manifest is missing: {}",
            corpus_manifest_path.display()
        ));
    }
    validate_bam_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "BAM endogenous truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.cases.is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth manifest `{}` must declare at least one case",
            manifest_path.display()
        ));
    }
    let mut sample_ids = BTreeSet::new();
    for case in &manifest.cases {
        if case.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "BAM endogenous truth manifest `{}` contains an empty sample_id",
                manifest_path.display()
            ));
        }
        if !sample_ids.insert(case.sample_id.clone()) {
            return Err(anyhow!(
                "BAM endogenous truth manifest repeats sample_id `{}`",
                case.sample_id
            ));
        }
        if case.method.trim().is_empty() {
            return Err(anyhow!(
                "BAM endogenous truth case `{}` must declare a non-empty method",
                case.sample_id
            ));
        }
        if case.host_reference_scope.trim().is_empty() {
            return Err(anyhow!(
                "BAM endogenous truth case `{}` must declare a non-empty host_reference_scope",
                case.sample_id
            ));
        }
        if !(0.0..=1.0).contains(&case.prealignment_fraction) {
            return Err(anyhow!(
                "BAM endogenous truth case `{}` prealignment_fraction {} must be within [0, 1]",
                case.sample_id,
                case.prealignment_fraction
            ));
        }
    }
    Ok(())
}

fn load_bam_endogenous_truth_bundle(expected_path: &Path) -> Result<BamEndogenousTruthBundle> {
    let raw = fs::read_to_string(expected_path)
        .with_context(|| format!("read {}", expected_path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", expected_path.display()))
}

fn validate_bundle_contract(
    manifest: &BamEndogenousTruthManifest,
    bundle: &BamEndogenousTruthBundle,
    expected_path: &Path,
) -> Result<()> {
    if bundle.schema_version != BAM_ENDOGENOUS_TRUTH_BUNDLE_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM endogenous truth bundle `{}` uses schema `{}` instead of `{}`",
            expected_path.display(),
            bundle.schema_version,
            BAM_ENDOGENOUS_TRUTH_BUNDLE_SCHEMA_VERSION
        ));
    }
    if bundle.fixture_id != manifest.fixture_id {
        return Err(anyhow!(
            "BAM endogenous truth bundle fixture_id `{}` does not match manifest fixture_id `{}`",
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
            "BAM endogenous truth bundle `{}` must contain samples {:?}",
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
    sample: &BamEndogenousTruthSample,
    expected_path: &Path,
) -> Result<()> {
    if sample.sample_id.trim().is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth bundle `{}` contains a sample with an empty sample_id",
            expected_path.display()
        ));
    }
    if sample.cohort.trim().is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` must declare a non-empty cohort",
            sample.sample_id
        ));
    }
    if sample.alignment_path.trim().is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` must declare a non-empty alignment_path",
            sample.sample_id
        ));
    }
    if sample.summary.schema_version != BAM_ENDOGENOUS_TRUTH_SUMMARY_SCHEMA_VERSION {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` uses summary schema `{}` instead of `{}`",
            sample.sample_id,
            sample.summary.schema_version,
            BAM_ENDOGENOUS_TRUTH_SUMMARY_SCHEMA_VERSION
        ));
    }
    if sample.summary.total_reads
        != sample.summary.mapped_reads.saturating_add(sample.summary.contaminant_reads)
    {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` total_reads {} does not equal mapped+contaminant {}",
            sample.sample_id,
            sample.summary.total_reads,
            sample.summary.mapped_reads + sample.summary.contaminant_reads
        ));
    }
    if sample.summary.retained_reads != sample.summary.mapped_reads {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` retained_reads {} must equal mapped_reads {}",
            sample.sample_id,
            sample.summary.retained_reads,
            sample.summary.mapped_reads
        ));
    }
    if sample.summary.endogenous_reads != sample.summary.retained_reads {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` endogenous_reads {} must equal retained_reads {}",
            sample.sample_id,
            sample.summary.endogenous_reads,
            sample.summary.retained_reads
        ));
    }
    let total_reads = sample.summary.total_reads as f64;
    let expected_contaminant_fraction = if sample.summary.total_reads > 0 {
        sample.summary.contaminant_reads as f64 / total_reads
    } else {
        0.0
    };
    let expected_retained_fraction = if sample.summary.total_reads > 0 {
        sample.summary.retained_reads as f64 / total_reads
    } else {
        0.0
    };
    if (sample.summary.contaminant_fraction - expected_contaminant_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` contaminant_fraction drifted",
            sample.sample_id
        ));
    }
    if (sample.summary.retained_fraction - expected_retained_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` retained_fraction drifted",
            sample.sample_id
        ));
    }
    if (sample.summary.endogenous_fraction - sample.summary.retained_fraction).abs() > 1e-9 {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` endogenous_fraction must equal retained_fraction",
            sample.sample_id
        ));
    }
    if sample.summary.count_provenance.trim().is_empty() {
        return Err(anyhow!(
            "BAM endogenous truth sample `{}` must declare non-empty count provenance",
            sample.sample_id
        ));
    }
    Ok(())
}

fn build_actual_truth_bundle(
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

fn load_bam_corpus_fixture_manifest_path(manifest_path: &Path) -> Result<BamCorpusFixtureManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
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
