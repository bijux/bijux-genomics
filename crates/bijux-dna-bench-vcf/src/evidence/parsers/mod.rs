//! Fixture coverage observes admitted bindings and preserves failed diagnostic artifacts.
mod admission;
mod envelope;
mod fixtures;
mod projection;
mod publication;
mod traversal;
pub use bijux_dna_bench_vcf_evidence::model::parsers::DEFAULT_VCF_PARSER_FIXTURE_COVERAGE_PATH;
pub use fixtures::coverage_status_label;
pub use projection::collect_vcf_parser_fixture_coverage_rows_from_retained;
pub use publication::render_vcf_parser_fixture_coverage;
pub const VCF_PARSER_FIXTURE_COVERAGE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_parser_fixture_coverage.v1";

pub mod failure;
