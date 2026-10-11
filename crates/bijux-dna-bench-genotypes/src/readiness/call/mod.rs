//! Calling readiness separates observation, surface policy and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfCallReadyReport, VcfCallReadyRow};
pub use service::render_vcf_call_ready;
pub const DEFAULT_VCF_CALL_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/call-ready.json";
pub(super) const VCF_CALL_READY_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_call_ready.v1";
pub(super) const VCF_CALL_STAGE_ID: &str = "vcf.call";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 4] =
    ["variant_count", "snp_count", "indel_count", "sample_count"];
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
