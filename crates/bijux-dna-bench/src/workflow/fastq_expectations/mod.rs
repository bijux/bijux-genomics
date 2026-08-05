//! Owner: bijux-dna-bench
//! Publication of exact FASTQ read-length expectation evidence.

mod rendering;
mod storage;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    validate_fastq_read_length_expectations, AdmittedFastqCorpus, BenchmarkFastqLayout,
    FastqReadLengthExpectationSample, FastqReadLengthExpectations,
    FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1,
};
use bijux_dna_science::evidence::{
    derive_fastq_read_length_truth, FastqInputLayout, FastqReadLengthTruth,
};

use self::rendering::{render_histogram, render_summary};
use self::storage::{ensure_directory_chain, ensure_immutable_file, verify_directory_closure};

const EXPECTATION_KIND: &str = "fastq-profile-read-lengths";

#[derive(Debug, Clone, PartialEq)]
pub struct FastqReadLengthExpectationPublication {
    pub root: PathBuf,
    pub manifest_path: PathBuf,
    pub expectations: FastqReadLengthExpectations,
    pub admitted_corpus: AdmittedFastqCorpus,
}

/// Derive and immutably publish exact read-length evidence for one admitted FASTQ corpus.
///
/// Every invocation re-admits and re-hashes the source corpus, independently derives science truth,
/// and accepts existing output only when every byte is identical. The manifest is published last.
///
/// # Errors
/// Returns an error when corpus admission or scientific derivation fails, a destination contains a
/// symlink or conflicting artifact, output closure differs, or the manifest is invalid.
pub fn ensure_fastq_read_length_expectations(
    study_root: &Path,
    data_index_path: &Path,
    corpus_id: &str,
    expected_sample_count: usize,
    expected_file_count: usize,
) -> Result<FastqReadLengthExpectationPublication> {
    let admitted = crate::repo::admit_fastq_benchmark_corpus(
        study_root,
        data_index_path,
        corpus_id,
        expected_sample_count,
        expected_file_count,
    )?;
    let study_root = std::fs::canonicalize(study_root)
        .with_context(|| format!("resolve STUDY root {}", study_root.display()))?;
    let root = study_root
        .join("data/evidence/benchmark-expectations")
        .join(&admitted.catalog_sha256)
        .join(EXPECTATION_KIND);
    let summaries_root = root.join("summaries");
    let histograms_root = root.join("histograms");
    for directory in [&root, &summaries_root, &histograms_root] {
        ensure_directory_chain(&study_root, directory)?;
    }

    let mut samples = Vec::with_capacity(admitted.samples.len());
    let mut summary_names = BTreeSet::new();
    let mut histogram_names = BTreeSet::new();
    for sample in &admitted.samples {
        let input_layout = science_layout(sample.input_layout);
        let paths = sample.files.iter().map(|file| PathBuf::from(&file.path)).collect::<Vec<_>>();
        let truth = derive_fastq_read_length_truth(&sample.sample_id, input_layout, &paths)
            .with_context(|| format!("derive read-length truth for {}", sample.sample_id))?;
        let summary_name = format!("{}.summary.tsv", sample.sample_id);
        let histogram_name = format!("{}.histogram.tsv", sample.sample_id);
        let summary_bytes = render_summary(&truth).into_bytes();
        let histogram_bytes = render_histogram(&truth).into_bytes();
        ensure_immutable_file(&summaries_root.join(&summary_name), &summary_bytes)?;
        ensure_immutable_file(&histograms_root.join(&histogram_name), &histogram_bytes)?;
        summary_names.insert(summary_name);
        histogram_names.insert(histogram_name);
        samples.push(expectation_sample(
            sample.input_layout,
            &truth,
            &summary_bytes,
            &histogram_bytes,
        ));
    }
    verify_directory_closure(&summaries_root, &summary_names)?;
    verify_directory_closure(&histograms_root, &histogram_names)?;
    let manifest_path = root.join("manifest.json");
    let root_names = match std::fs::symlink_metadata(&manifest_path) {
        Ok(_) => BTreeSet::from([
            "histograms".to_string(),
            "manifest.json".to_string(),
            "summaries".to_string(),
        ]),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            BTreeSet::from(["histograms".to_string(), "summaries".to_string()])
        }
        Err(error) => {
            return Err(error).with_context(|| {
                format!("inspect expectation manifest {}", manifest_path.display())
            });
        }
    };
    verify_directory_closure(&root, &root_names)?;

    let expectations = FastqReadLengthExpectations {
        schema_version: FASTQ_READ_LENGTH_EXPECTATIONS_SCHEMA_V1.to_string(),
        corpus_id: admitted.corpus_id.clone(),
        sample_count: admitted.sample_count,
        file_count: admitted.file_count,
        data_index_sha256: admitted.data_index_sha256.clone(),
        catalog_sha256: admitted.catalog_sha256.clone(),
        samples,
    };
    validate_fastq_read_length_expectations(&expectations)
        .context("validate FASTQ read-length expectation publication")?;
    let mut manifest_bytes = serde_json::to_vec_pretty(&expectations)
        .context("serialize FASTQ read-length expectation manifest")?;
    manifest_bytes.push(b'\n');
    ensure_immutable_file(&manifest_path, &manifest_bytes)?;
    verify_directory_closure(
        &root,
        &BTreeSet::from([
            "histograms".to_string(),
            "manifest.json".to_string(),
            "summaries".to_string(),
        ]),
    )?;
    Ok(FastqReadLengthExpectationPublication {
        root,
        manifest_path,
        expectations,
        admitted_corpus: admitted,
    })
}

fn science_layout(layout: BenchmarkFastqLayout) -> FastqInputLayout {
    match layout {
        BenchmarkFastqLayout::SingleEnd => FastqInputLayout::SingleEnd,
        BenchmarkFastqLayout::PairedEnd => FastqInputLayout::PairedEnd,
    }
}

fn expectation_sample(
    input_layout: BenchmarkFastqLayout,
    truth: &FastqReadLengthTruth,
    summary_bytes: &[u8],
    histogram_bytes: &[u8],
) -> FastqReadLengthExpectationSample {
    FastqReadLengthExpectationSample {
        sample_id: truth.sample_id.clone(),
        input_layout,
        file_count: truth.file_count,
        read_count: truth.read_count,
        minimum_read_length: truth.minimum_read_length,
        mean_read_length: truth.mean_read_length,
        median_read_length: truth.median_read_length,
        maximum_read_length: truth.maximum_read_length,
        distinct_read_lengths: truth.distinct_read_lengths,
        summary_path: format!("summaries/{}.summary.tsv", truth.sample_id),
        summary_sha256: bijux_dna_infra::hash_bytes_sha256(summary_bytes),
        histogram_path: format!("histograms/{}.histogram.tsv", truth.sample_id),
        histogram_sha256: bijux_dna_infra::hash_bytes_sha256(histogram_bytes),
        exact_rows_sha256: truth.exact_rows_sha256.clone(),
    }
}
