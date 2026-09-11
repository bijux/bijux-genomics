//! Fixture coverage observes admitted bindings and preserves failed diagnostic artifacts.
mod admission;
mod envelope;
mod fixtures;
mod projection;
mod publication;
mod traversal;
pub use fixtures::coverage_status_label;
pub use projection::collect_vcf_parser_fixture_coverage_rows_from_retained;
pub use publication::render_vcf_parser_fixture_coverage;
pub const DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-parser-fixture-coverage.tsv";
pub const VCF_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_parser_fixture_coverage.v1";
