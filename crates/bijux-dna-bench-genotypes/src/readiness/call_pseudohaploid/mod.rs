//! Calling readiness separates observation, surface policy and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfCallPseudohaploidReadyReport, VcfCallPseudohaploidReadyRow};
pub use service::render_vcf_call_pseudohaploid_ready;
pub const DEFAULT_VCF_CALL_PSEUDOHAPLOID_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/call-pseudohaploid-ready.json";
pub(super) const VCF_CALL_PSEUDOHAPLOID_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_call_pseudohaploid_ready.v1";
pub(super) const VCF_CALL_PSEUDOHAPLOID_STAGE_ID: &str = "vcf.call_pseudohaploid";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 7] = [
    "target_sites",
    "covered_sites",
    "called_sites",
    "missing_sites",
    "sampling_policy",
    "random_seed",
    "sample_count",
];
pub(super) const REQUIRED_SAMPLING_POLICY: &str = "deterministic_first_allele_projection";
pub(super) const REQUIRED_SEED_USAGE: &str = "control_seed_for_replay_proof";
pub(super) const REQUIRED_RANDOM_SEED: u64 = 73;
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
