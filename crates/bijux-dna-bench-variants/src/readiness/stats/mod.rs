//! Stage readiness separates observation, surface policy and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfStatsReadyReport, VcfStatsReadyRow};
pub use service::render_vcf_stats_ready;
pub const DEFAULT_VCF_STATS_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/stats-ready.json";
pub(super) const VCF_STATS_READY_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_stats_ready.v1";
pub(super) const VCF_STATS_STAGE_ID: &str = "vcf.stats";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 7] = [
    "variant_count",
    "snp_count",
    "indel_count",
    "transition_count",
    "transversion_count",
    "ti_tv",
    "sample_count",
];
pub(super) const REQUIRED_VARIANT_COUNT: u64 = 4;
pub(super) const REQUIRED_SNP_COUNT: u64 = 3;
pub(super) const REQUIRED_INDEL_COUNT: u64 = 1;
pub(super) const REQUIRED_TRANSITION_COUNT: u64 = 2;
pub(super) const REQUIRED_TRANSVERSION_COUNT: u64 = 1;
pub(super) const REQUIRED_TI_TV: f64 = 2.0;
pub(super) const REQUIRED_SAMPLE_COUNT: u64 = 2;
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
