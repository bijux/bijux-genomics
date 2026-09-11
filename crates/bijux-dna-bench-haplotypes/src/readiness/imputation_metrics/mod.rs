//! Stage readiness separates scientific evidence, projection and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfImputationMetricsReadyReport, VcfImputationMetricsReadyRow};
pub use service::render_vcf_imputation_metrics_ready;
pub const DEFAULT_VCF_IMPUTATION_METRICS_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/imputation-metrics-ready.json";
pub(super) const VCF_IMPUTATION_METRICS_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_imputation_metrics_ready.v1";
pub(super) const VCF_IMPUTATION_METRICS_STAGE_ID: &str = "vcf.imputation_metrics";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 11] = [
    "status",
    "concordance",
    "mean_info_score",
    "r2_available",
    "dosage_r2",
    "low_confidence_sites",
    "masked_truth_sites",
    "missing_quality_fields",
    "sample_count",
    "sample_ids",
    "variant_count",
];
pub(super) const REQUIRED_CONCORDANCE: f64 = 1.0;
pub(super) const REQUIRED_MEAN_INFO_SCORE: f64 = 0.825;
pub(super) const REQUIRED_DOSAGE_R2: f64 = 0.775;
pub(super) const REQUIRED_LOW_CONFIDENCE_SITES: u64 = 1;
pub(super) const REQUIRED_MASKED_TRUTH_SITES: u64 = 1;
pub(super) const REQUIRED_STATUS: &str = "complete";
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
pub(super) const FLOAT_TOLERANCE: f64 = 1e-9;

#[cfg(test)]
#[allow(clippy::expect_used)]
#[path = "tests.rs"]
mod tests;
