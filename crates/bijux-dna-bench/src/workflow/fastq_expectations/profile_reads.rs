//! Owner: bijux-dna-bench
//! Immutable read-count and base-count truth derived from admitted FASTQ evidence.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_fastq_read_length_expectations, AdmittedFastqCorpus, BenchmarkFastqLayout,
};
use serde::Serialize;

use super::storage::{ensure_directory_chain, ensure_immutable_file, verify_directory_closure};
use super::FastqReadLengthExpectationPublication;

const PROFILE_HEADER: &str = "sample_id\tinput_layout\tfile_count\tread_count\tbase_count";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FastqProfileReadsExpectationSample {
    pub sample_id: String,
    pub input_layout: BenchmarkFastqLayout,
    pub file_count: u64,
    pub read_count: u64,
    pub base_count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FastqProfileReadsExpectationPublication {
    pub root: PathBuf,
    pub profile_path: PathBuf,
    pub provenance_path: PathBuf,
    pub admitted_corpus: AdmittedFastqCorpus,
    pub samples: Vec<FastqProfileReadsExpectationSample>,
}

#[derive(Serialize)]
struct ProfileProvenance<'a> {
    schema_version: &'static str,
    corpus_id: &'a str,
    sample_count: u64,
    file_count: u64,
    data_index_sha256: &'a str,
    catalog_sha256: &'a str,
    read_length_derivation_sha256: &'a str,
    expected_profile_sha256: String,
}

/// Publish exact per-sample read and base counts from independently decoded FASTQ evidence.
///
/// # Errors
/// Returns an error when read-length evidence is invalid, histogram artifacts differ from their
/// digests, counts overflow or disagree, or immutable publication bytes conflict.
pub fn ensure_fastq_profile_reads_expectations(
    study_root: &Path,
    read_lengths: &FastqReadLengthExpectationPublication,
) -> Result<FastqProfileReadsExpectationPublication> {
    validate_fastq_read_length_expectations(&read_lengths.expectations)
        .context("validate source FASTQ read-length expectations")?;
    let study_root = study_root
        .canonicalize()
        .with_context(|| format!("resolve STUDY root {}", study_root.display()))?;
    let read_length_root = read_lengths
        .root
        .canonicalize()
        .with_context(|| format!("resolve read-length evidence {}", read_lengths.root.display()))?;
    if !read_length_root.starts_with(&study_root) {
        bail!("read-length evidence escapes STUDY");
    }
    let derivation_root =
        read_length_root.parent().context("read-length evidence has no derivation root")?;
    let root = derivation_root.join("fastq-profile-reads");
    ensure_directory_chain(&study_root, &root)?;

    let mut samples = Vec::with_capacity(read_lengths.expectations.samples.len());
    for expected in &read_lengths.expectations.samples {
        let histogram_path = read_length_root.join(&expected.histogram_path);
        let (read_count, base_count) =
            histogram_counts(&histogram_path, &expected.sample_id, &expected.histogram_sha256)?;
        if read_count != expected.read_count {
            bail!("FASTQ histogram read count differs for {}", expected.sample_id);
        }
        samples.push(FastqProfileReadsExpectationSample {
            sample_id: expected.sample_id.clone(),
            input_layout: expected.input_layout,
            file_count: expected.file_count,
            read_count: expected.read_count,
            base_count,
        });
    }

    let profile_bytes = render_profile(&samples).into_bytes();
    let profile_path = root.join("expected-profile.tsv");
    ensure_immutable_file(&profile_path, &profile_bytes)?;
    let provenance = ProfileProvenance {
        schema_version: "bijux.benchmark.fastq.profile_reads.expectations.v1",
        corpus_id: &read_lengths.expectations.corpus_id,
        sample_count: read_lengths.expectations.sample_count,
        file_count: read_lengths.expectations.file_count,
        data_index_sha256: &read_lengths.expectations.data_index_sha256,
        catalog_sha256: &read_lengths.expectations.catalog_sha256,
        read_length_derivation_sha256: &read_lengths.expectations.derivation_sha256,
        expected_profile_sha256: bijux_dna_infra::hash_bytes_sha256(&profile_bytes),
    };
    let mut provenance_bytes =
        serde_json::to_vec_pretty(&provenance).context("serialize FASTQ profile provenance")?;
    provenance_bytes.push(b'\n');
    let provenance_path = root.join("provenance.json");
    ensure_immutable_file(&provenance_path, &provenance_bytes)?;
    verify_directory_closure(
        &root,
        &BTreeSet::from(["expected-profile.tsv".to_string(), "provenance.json".to_string()]),
    )?;
    Ok(FastqProfileReadsExpectationPublication {
        root,
        profile_path,
        provenance_path,
        admitted_corpus: read_lengths.admitted_corpus.clone(),
        samples,
    })
}

fn histogram_counts(path: &Path, sample_id: &str, expected_sha256: &str) -> Result<(u64, u64)> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspect FASTQ histogram {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("FASTQ histogram is not a regular non-symlink file for {sample_id}");
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, 64 * 1024 * 1024)
        .with_context(|| format!("read FASTQ histogram {}", path.display()))?;
    if bijux_dna_infra::hash_bytes_sha256(&bytes) != expected_sha256 {
        bail!("FASTQ histogram digest differs for {sample_id}");
    }
    let content = std::str::from_utf8(&bytes)
        .with_context(|| format!("decode FASTQ histogram {}", path.display()))?;
    let mut lines = content.lines();
    if lines.next() != Some("sample_id\tread_length\tcount") {
        bail!("FASTQ histogram header is invalid for {sample_id}");
    }
    let mut reads = 0_u64;
    let mut bases = 0_u64;
    for line in lines {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 3 || fields[0] != sample_id {
            bail!("FASTQ histogram row is invalid for {sample_id}");
        }
        let length = fields[1]
            .parse::<u64>()
            .with_context(|| format!("parse FASTQ length for {sample_id}"))?;
        let count = fields[2]
            .parse::<u64>()
            .with_context(|| format!("parse FASTQ count for {sample_id}"))?;
        if length == 0 || count == 0 {
            bail!("FASTQ histogram contains zero values for {sample_id}");
        }
        reads = reads.checked_add(count).context("FASTQ profile read count overflow")?;
        bases = bases
            .checked_add(length.checked_mul(count).context("FASTQ profile base count overflow")?)
            .context("FASTQ profile base count overflow")?;
    }
    if reads == 0 {
        bail!("FASTQ histogram is empty for {sample_id}");
    }
    Ok((reads, bases))
}

fn render_profile(samples: &[FastqProfileReadsExpectationSample]) -> String {
    let mut output = String::from(PROFILE_HEADER);
    output.push('\n');
    for sample in samples {
        let layout = match sample.input_layout {
            BenchmarkFastqLayout::SingleEnd => "single_end",
            BenchmarkFastqLayout::PairedEnd => "paired_end",
        };
        output.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\n",
            sample.sample_id, layout, sample.file_count, sample.read_count, sample.base_count
        ));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::histogram_counts;

    #[test]
    fn histogram_counts_bases_without_float_rounding() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("fastq-profile-counts")?;
        let path = temporary.path().join("histogram.tsv");
        std::fs::write(&path, b"sample_id\tread_length\tcount\nERR1\t2\t3\nERR1\t5\t2\n")?;
        let digest = bijux_dna_infra::hash_file_sha256(&path)?;
        assert_eq!(histogram_counts(&path, "ERR1", &digest)?, (5, 16));
        Ok(())
    }
}
