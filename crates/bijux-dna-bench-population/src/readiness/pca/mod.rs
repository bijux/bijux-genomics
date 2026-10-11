//! PCA readiness separates admitted adapter observations from scientific assessment and publication.
mod admission;
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod science;
mod service;
pub use model::{VcfPcaReadyReport, VcfPcaReadyRow};
pub use service::render_vcf_pca_ready;
pub const DEFAULT_VCF_PCA_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/pca-ready.json";
pub(super) const VCF_PCA_READY_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_pca_ready.v1";
pub(super) const VCF_PCA_STAGE_ID: &str = "vcf.pca";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 9] = [
    "sample_count",
    "variant_count",
    "excluded_samples",
    "unexpected_samples",
    "eigenvalues",
    "status",
    "rows",
    "execution_mode",
    "tool_ok",
];
pub(super) const REQUIRED_VARIANT_COUNT: u64 = 2;
pub(super) const REQUIRED_SAMPLE_COUNT: u64 = 4;
pub(super) const REQUIRED_SAMPLE_IDS: [&str; 4] = ["sample_a", "sample_b", "sample_c", "sample_d"];
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
pub(super) const VCF_PCA_READY_LOCK_PATH: &str =
    "artifacts/bench-readiness/vcf-pca-ready/render.lock";
