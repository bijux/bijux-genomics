//! Calling readiness separates observation, surface policy and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfCallGlReadyReport, VcfCallGlReadyRow};
pub use service::render_vcf_call_gl_ready;
pub const DEFAULT_VCF_CALL_GL_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/call-gl-ready.json";
pub(super) const VCF_CALL_GL_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_call_gl_ready.v1";
pub(super) const VCF_CALL_GL_STAGE_ID: &str = "vcf.call_gl";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 5] = [
    "likelihood_field",
    "sites_with_likelihoods",
    "samples_with_likelihoods",
    "missing_likelihoods",
    "sample_count",
];
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
