mod model;
pub use model::{
    FastqAdapterCoverageKind, FastqBenchmarkStatus, FastqCommandAdapterCoverageReport,
    FastqCommandAdapterCoverageRow, FastqReadinessGapKind,
};
pub const DEFAULT_FASTQ_COMMAND_ADAPTER_COVERAGE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/fastq/fastq-command-adapter-coverage.tsv";
pub const FASTQ_COMMAND_ADAPTER_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_command_adapter_coverage.v1";

mod projection;
mod serialization;
pub use projection::collect_fastq_command_adapter_coverage_rows;
mod service;
pub use service::render_fastq_command_adapter_coverage;
