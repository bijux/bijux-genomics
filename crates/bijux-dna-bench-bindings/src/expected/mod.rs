//! Expected result admission, artifact roles, paths and publication.
mod admission;
mod bam;
mod fastq;
mod index;
mod models;
mod projection;
mod publication;
mod service;
mod tsv;
pub use models::{ExpectedBenchmarkResultRow, ExpectedBenchmarkResultsReport};
pub use publication::render_expected_benchmark_results;
pub use service::collect_expected_benchmark_result_rows;
pub const EXPECTED_BENCHMARK_RESULTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.expected_benchmark_results.v1";
