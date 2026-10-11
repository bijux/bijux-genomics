//! Synthetic missing-result recovery keeps each expected row visible.
mod admission;
mod contract;
mod fixture;
mod model;
mod paths;
mod projection;
mod service;
pub use model::{VcfMissingResultReport, VcfMissingResultReportRow, VcfMissingResultStatus};
pub use service::render_vcf_missing_result_report;
pub const DEFAULT_VCF_MISSING_RESULT_REPORT_TEST_PATH: &str =
    "artifacts/benchmark-smoke/readiness/missing-result/vcf/report.json";
pub const DEFAULT_VCF_MISSING_RESULT_REPORT_FIXTURE_ROOT: &str =
    "artifacts/benchmark-smoke/readiness/missing-result/vcf/results";
pub const VCF_MISSING_RESULT_REPORT_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_missing_result_report.v1";
