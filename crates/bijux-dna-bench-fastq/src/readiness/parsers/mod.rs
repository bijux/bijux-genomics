mod model;
pub use model::{FastqParserCoverageKind, FastqParserCoverageReport, FastqParserCoverageRow};
pub const DEFAULT_FASTQ_PARSER_COVERAGE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/fastq/fastq-parser-coverage.tsv";
pub const FASTQ_PARSER_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_parser_coverage.v1";

mod projection;
mod serialization;
pub use projection::collect_fastq_parser_coverage_rows;
mod service;
pub use service::render_fastq_parser_coverage;
