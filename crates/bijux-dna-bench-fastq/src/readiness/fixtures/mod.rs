mod model;
pub use model::{
    FastqParserFixtureCoverageReport, FastqParserFixtureCoverageRow,
    FastqParserFixtureCoverageStatus,
};
pub const DEFAULT_FASTQ_PARSER_FIXTURE_COVERAGE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/fastq/fastq-parser-fixture-coverage.tsv";
pub const FASTQ_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.fastq_parser_fixture_coverage.v1";
const FIXTURE_REFERENCE_KIND: &str = "fixture_case";

mod parsing;
mod projection;
mod serialization;
pub use projection::collect_fastq_parser_fixture_coverage_rows;
pub use serialization::coverage_status_label;
mod service;
pub use service::render_fastq_parser_fixture_coverage;
