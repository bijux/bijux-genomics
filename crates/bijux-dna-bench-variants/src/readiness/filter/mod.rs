//! Stage readiness separates observation, surface policy and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfFilterReadyReport, VcfFilterReadyRow};
pub use service::render_vcf_filter_ready;
pub const DEFAULT_VCF_FILTER_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/filter-ready.json";
pub(super) const VCF_FILTER_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_filter_ready.v1";
pub(super) const VCF_FILTER_STAGE_ID: &str = "vcf.filter";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 8] = [
    "input_variants",
    "pass_variants",
    "failed_variants",
    "filter_ids",
    "depth_threshold",
    "quality_threshold",
    "missingness_threshold",
    "sample_count",
];
pub(super) const REQUIRED_FILTER_IDS: [&str; 4] = ["HIGH_MISSING", "LOWQUAL", "LOW_DP", "LOW_MQ"];
pub(super) const REQUIRED_DEPTH_THRESHOLD: f64 = 8.0;
pub(super) const REQUIRED_QUALITY_THRESHOLD: f64 = 30.0;
pub(super) const REQUIRED_MISSINGNESS_THRESHOLD: f64 = 0.2;
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
