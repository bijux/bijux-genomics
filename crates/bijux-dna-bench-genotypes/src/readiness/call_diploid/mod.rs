//! Calling readiness separates observation, surface policy and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfCallDiploidReadyReport, VcfCallDiploidReadyRow};
pub use service::render_vcf_call_diploid_ready;
pub const DEFAULT_VCF_CALL_DIPLOID_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/call-diploid-ready.json";
pub(super) const VCF_CALL_DIPLOID_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_call_diploid_ready.v1";
pub(super) const VCF_CALL_DIPLOID_STAGE_ID: &str = "vcf.call_diploid";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 7] = [
    "ploidy",
    "called_genotypes",
    "heterozygous_count",
    "homozygous_ref_count",
    "homozygous_alt_count",
    "missing_count",
    "sample_count",
];
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
