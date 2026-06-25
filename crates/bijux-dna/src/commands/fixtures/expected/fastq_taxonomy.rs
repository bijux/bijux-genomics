#![cfg_attr(test, allow(clippy::expect_used))]

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use crate::commands::benchmark::local_corpus_fixture::edna::{
    load_edna_corpus_fixture_manifest_path, load_validated_edna_expected_taxa_rows,
    validate_edna_corpus_fixture_manifest_contract,
};
use crate::commands::benchmark::local_taxonomy_output_judgment::{
    judge_edna_taxonomy_outputs_with_expected_rows, LocalTaxonomyObservedReportArg,
};

pub(crate) const FASTQ_TAXONOMY_TRUTH_FIXTURE_ID: &str = "fastq-taxonomy-truth";
pub(crate) const FASTQ_TAXONOMY_TRUTH_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.fastq_taxonomy_truth.v1";
const FASTQ_TAXONOMY_TRUTH_VALIDATION_SCHEMA_VERSION: &str =
    "bijux.bench.fastq_taxonomy_truth.validation.v1";

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct FastqTaxonomyTruthManifest {
    schema_version: String,
    fixture_id: String,
    description: String,
    corpus_manifest_path: PathBuf,
    expected_taxa_path: PathBuf,
    source_paths: Vec<PathBuf>,
    samples: Vec<FastqTaxonomyTruthSampleManifest>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct FastqTaxonomyTruthSampleManifest {
    sample_id: String,
    observed_report_path: PathBuf,
    expected_unclassified_percent: f64,
    expected_unclassified_read_count: u64,
    expected_false_positive_count: usize,
    expected_unexpected_taxa: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct FastqTaxonomyTruthValidationReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_id: String,
    pub(crate) manifest_path: String,
    pub(crate) expected_taxa_path: String,
    pub(crate) validated_sample_count: usize,
    pub(crate) validated_taxa_row_count: usize,
    pub(crate) valid: bool,
    pub(crate) checked_samples: Vec<String>,
}

pub(crate) fn validate_fastq_taxonomy_truth_manifest_path(
    repo_root: &Path,
    manifest_path: &Path,
) -> Result<FastqTaxonomyTruthValidationReport> {
    let manifest = load_fastq_taxonomy_truth_manifest_path(manifest_path)?;
    validate_manifest_contract(repo_root, &manifest, manifest_path)?;

    let fixture_root = manifest_path.parent().ok_or_else(|| {
        anyhow!(
            "FASTQ taxonomy truth manifest has no parent directory: {}",
            manifest_path.display()
        )
    })?;
    let expected_taxa_path = resolve_fixture_path(fixture_root, &manifest.expected_taxa_path);
    if !expected_taxa_path.is_file() {
        return Err(anyhow!(
            "FASTQ taxonomy truth expected taxa table is missing: {}",
            expected_taxa_path.display()
        ));
    }

    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    let corpus_manifest = load_edna_corpus_fixture_manifest_path(&corpus_manifest_path)?;
    validate_edna_corpus_fixture_manifest_contract(&corpus_manifest)?;
    let expected_rows =
        load_validated_edna_expected_taxa_rows(&corpus_manifest, &expected_taxa_path)?;
    let report_args = manifest
        .samples
        .iter()
        .map(|sample| LocalTaxonomyObservedReportArg {
            sample_id: sample.sample_id.clone(),
            report_path: resolve_repo_relative_path(repo_root, &sample.observed_report_path),
        })
        .collect::<Vec<_>>();
    let judgment = judge_edna_taxonomy_outputs_with_expected_rows(
        repo_root,
        &corpus_manifest_path,
        &corpus_manifest,
        &expected_taxa_path,
        &expected_rows,
        &report_args,
    )?;

    validate_sample_expectations(&manifest, &judgment.samples)?;

    let checked_samples =
        judgment.samples.iter().map(|sample| sample.sample_id.clone()).collect::<Vec<_>>();
    let validated_sample_count = checked_samples.len();

    Ok(FastqTaxonomyTruthValidationReport {
        schema_version: FASTQ_TAXONOMY_TRUTH_VALIDATION_SCHEMA_VERSION,
        fixture_id: manifest.fixture_id,
        manifest_path: path_relative_to_repo(repo_root, manifest_path),
        expected_taxa_path: path_relative_to_repo(repo_root, &expected_taxa_path),
        validated_sample_count,
        validated_taxa_row_count: expected_rows.len(),
        valid: judgment.valid,
        checked_samples,
    })
}

fn load_fastq_taxonomy_truth_manifest_path(
    manifest_path: &Path,
) -> Result<FastqTaxonomyTruthManifest> {
    let raw = fs::read_to_string(manifest_path)
        .with_context(|| format!("read {}", manifest_path.display()))?;
    toml::from_str(&raw).with_context(|| format!("parse {}", manifest_path.display()))
}

fn validate_manifest_contract(
    repo_root: &Path,
    manifest: &FastqTaxonomyTruthManifest,
    manifest_path: &Path,
) -> Result<()> {
    if manifest.schema_version != FASTQ_TAXONOMY_TRUTH_MANIFEST_SCHEMA_VERSION {
        return Err(anyhow!(
            "FASTQ taxonomy truth manifest `{}` uses schema `{}` instead of `{}`",
            manifest_path.display(),
            manifest.schema_version,
            FASTQ_TAXONOMY_TRUTH_MANIFEST_SCHEMA_VERSION
        ));
    }
    if manifest.fixture_id != FASTQ_TAXONOMY_TRUTH_FIXTURE_ID {
        return Err(anyhow!(
            "FASTQ taxonomy truth manifest fixture_id `{}` must equal `{}`",
            manifest.fixture_id,
            FASTQ_TAXONOMY_TRUTH_FIXTURE_ID
        ));
    }
    if manifest.description.trim().is_empty() {
        return Err(anyhow!(
            "FASTQ taxonomy truth manifest `{}` must declare a description",
            manifest_path.display()
        ));
    }
    let corpus_manifest_path =
        resolve_repo_relative_path(repo_root, &manifest.corpus_manifest_path);
    if !corpus_manifest_path.is_file() {
        return Err(anyhow!(
            "FASTQ taxonomy truth corpus manifest is missing: {}",
            corpus_manifest_path.display()
        ));
    }
    if manifest.source_paths.is_empty() {
        return Err(anyhow!(
            "FASTQ taxonomy truth manifest `{}` must declare governed source paths",
            manifest_path.display()
        ));
    }
    for source_path in &manifest.source_paths {
        let resolved = resolve_repo_relative_path(repo_root, source_path);
        if !resolved.is_file() {
            return Err(anyhow!(
                "FASTQ taxonomy truth manifest source path is missing: {}",
                resolved.display()
            ));
        }
    }
    if manifest.samples.is_empty() {
        return Err(anyhow!(
            "FASTQ taxonomy truth manifest `{}` must declare at least one sample",
            manifest_path.display()
        ));
    }
    let mut sample_ids = BTreeSet::new();
    for sample in &manifest.samples {
        if sample.sample_id.trim().is_empty() {
            return Err(anyhow!(
                "FASTQ taxonomy truth manifest `{}` contains an empty sample_id",
                manifest_path.display()
            ));
        }
        if !sample_ids.insert(sample.sample_id.clone()) {
            return Err(anyhow!(
                "FASTQ taxonomy truth manifest repeats sample_id `{}`",
                sample.sample_id
            ));
        }
        if !sample.expected_unclassified_percent.is_finite()
            || sample.expected_unclassified_percent.is_sign_negative()
        {
            return Err(anyhow!(
                "FASTQ taxonomy truth sample `{}` must declare a non-negative finite expected_unclassified_percent",
                sample.sample_id
            ));
        }
        let report_path = resolve_repo_relative_path(repo_root, &sample.observed_report_path);
        if !report_path.is_file() {
            return Err(anyhow!(
                "FASTQ taxonomy truth sample `{}` observed report is missing: {}",
                sample.sample_id,
                report_path.display()
            ));
        }
    }
    Ok(())
}

fn validate_sample_expectations(
    manifest: &FastqTaxonomyTruthManifest,
    sample_judgments: &[crate::commands::benchmark::local_taxonomy_output_judgment::LocalTaxonomySampleJudgment],
) -> Result<()> {
    if sample_judgments.len() != manifest.samples.len() {
        return Err(anyhow!(
            "FASTQ taxonomy truth validated {} samples but manifest declares {}",
            sample_judgments.len(),
            manifest.samples.len()
        ));
    }
    for sample in &manifest.samples {
        let judgment = sample_judgments
            .iter()
            .find(|judgment| judgment.sample_id == sample.sample_id)
            .ok_or_else(|| {
                anyhow!("missing taxonomy judgment for sample `{}`", sample.sample_id)
            })?;
        if !judgment.valid {
            return Err(anyhow!(
                "FASTQ taxonomy truth judgment is invalid for sample `{}`",
                sample.sample_id
            ));
        }
        if (judgment.observed_unclassified_percent - sample.expected_unclassified_percent).abs()
            > 1e-9
        {
            return Err(anyhow!(
                "FASTQ taxonomy truth sample `{}` observed unclassified percent {} but expected {}",
                sample.sample_id,
                judgment.observed_unclassified_percent,
                sample.expected_unclassified_percent
            ));
        }
        if judgment.observed_unclassified_read_count != sample.expected_unclassified_read_count {
            return Err(anyhow!(
                "FASTQ taxonomy truth sample `{}` observed {} unclassified reads but expected {}",
                sample.sample_id,
                judgment.observed_unclassified_read_count,
                sample.expected_unclassified_read_count
            ));
        }
        if judgment.false_positive_count != sample.expected_false_positive_count {
            return Err(anyhow!(
                "FASTQ taxonomy truth sample `{}` observed {} false positives but expected {}",
                sample.sample_id,
                judgment.false_positive_count,
                sample.expected_false_positive_count
            ));
        }
        let unexpected_taxa = judgment
            .unexpected_taxa
            .iter()
            .map(|entry| entry.name.as_str())
            .collect::<BTreeSet<_>>();
        let expected_unexpected_taxa =
            sample.expected_unexpected_taxa.iter().map(String::as_str).collect::<BTreeSet<_>>();
        if unexpected_taxa != expected_unexpected_taxa {
            return Err(anyhow!(
                "FASTQ taxonomy truth sample `{}` unexpected taxa drifted",
                sample.sample_id
            ));
        }
    }
    Ok(())
}

fn resolve_fixture_path(fixture_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        fixture_root.join(path)
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
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
