//! Calling readiness separates observation, surface policy and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfGlPropagationReadyReport, VcfGlPropagationReadyRow};
pub use service::render_vcf_gl_propagation_ready;
pub const DEFAULT_VCF_GL_PROPAGATION_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/gl-propagation-ready.json";
pub(super) const VCF_GL_PROPAGATION_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_gl_propagation_ready.v1";
pub(super) const VCF_GL_PROPAGATION_STAGE_ID: &str = "vcf.gl_propagation";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 6] = [
    "input_likelihood_fields",
    "output_likelihood_fields",
    "lost_fields",
    "site_count_before",
    "site_count_after",
    "sample_count",
];
pub(super) const REQUIRED_LIKELIHOOD_FIELDS: [&str; 3] = ["GL", "GP", "PL"];
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
