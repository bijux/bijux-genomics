//! Preserve the benchmark error taxonomy at the corpus-run compatibility surface.
use crate::{BenchError, BenchmarkCorpusRunManifest};
/// Validate one persisted corpus benchmark run.
///
/// # Errors
/// Preserves canonical corpus-run validation failures as the public benchmark error.
pub fn validate_benchmark_local_corpus_run(
    manifest: &BenchmarkCorpusRunManifest,
) -> Result<(), BenchError> {
    bijux_dna_bench_corpus::run::validate_benchmark_local_corpus_run(manifest)
        .map_err(|error| BenchError::InvalidPolicy(error.into_reason()))
}
