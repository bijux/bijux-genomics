//! Governed malformed-output cases, parser execution and diagnostic publication.
mod cases;
mod fixtures;
mod model;
mod mutation;
mod parsing;
mod projection;
mod service;
pub use model::{VcfParserFailureTestRow, VcfParserFailureTestsReport};
pub use service::render_vcf_parser_failure_audit;
pub const DEFAULT_VCF_PARSER_FAILURE_TESTS_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/vcf-parser-failure-tests.json";
pub const VCF_PARSER_FAILURE_TESTS_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_parser_failure_audit.v1";
