//! Persisted corpus-run records, semantic admission and publication time formatting.
mod model;
mod time;
mod validation;
pub use model::{
    benchmark_local_corpus_run_schema, BenchmarkCorpusRunManifest, BenchmarkCorpusSampleRun,
};
pub use time::format_generated_at_utc;
pub use validation::validate_benchmark_local_corpus_run;
pub const BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1: &str = "bijux.fastq.corpus_benchmark_run.v1";
/// A corpus-run record violates its persisted contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CorpusRunError {
    reason: String,
}
impl CorpusRunError {
    fn invalid(reason: String) -> Self {
        Self { reason }
    }
    /// Preserve the diagnostic reason when adapting an existing public error taxonomy.
    #[must_use]
    pub fn into_reason(self) -> String {
        self.reason
    }
}
impl std::fmt::Display for CorpusRunError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "invalid policy: {}", self.reason)
    }
}
impl std::error::Error for CorpusRunError {}
