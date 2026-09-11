//! Admission of persisted manifest identity and individual sample outcomes.
mod fields;
mod manifest;
mod samples;
use super::{BenchmarkCorpusRunManifest, CorpusRunError};
/// Validate one FASTQ corpus benchmark run manifest.
///
/// # Errors
/// Rejects invalid identity, cardinality, resources, timestamps or sample outcomes.
pub fn validate_benchmark_local_corpus_run(
    manifest: &BenchmarkCorpusRunManifest,
) -> Result<(), CorpusRunError> {
    manifest::validate_manifest_contract(manifest)?;
    samples::validate_sample_runs(manifest)
}
