//! Stage readiness separates observation, scientific policy and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfAdmixtureReadyReport, VcfAdmixtureReadyRow};
pub use service::render_vcf_admixture_ready;
pub const DEFAULT_VCF_ADMIXTURE_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/admixture-ready.json";
pub(super) const VCF_ADMIXTURE_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_admixture_ready.v1";
pub(super) const VCF_ADMIXTURE_STAGE_ID: &str = "vcf.admixture";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 12] = [
    "selected_k",
    "sample_count",
    "population_count",
    "status",
    "cluster_headers",
    "maf_summary",
    "rows",
    "variant_count",
    "sample_missingness",
    "variant_missingness",
    "execution_mode",
    "tool_ok",
];
pub(super) const REQUIRED_STATUS: &str = "complete";
pub(super) const REQUIRED_SELECTED_K: usize = 2;
pub(super) const REQUIRED_SAMPLE_COUNT: u64 = 4;
pub(super) const REQUIRED_POPULATION_COUNT: u64 = 2;
pub(super) const REQUIRED_SAMPLE_IDS: [&str; 4] = ["sample_a", "sample_b", "sample_c", "sample_d"];
pub(super) const REQUIRED_CLUSTER_HEADERS: [&str; 2] = ["cluster_1", "cluster_2"];
pub(super) const GOVERNED_STAGE_CONTRACT_SMOKE_TOOL_ID: &str = "plink2";
pub(super) const ANCESTRY_SUM_TOLERANCE: f64 = 1e-6;
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
pub(super) const VCF_ADMIXTURE_READY_LOCK_PATH: &str =
    "artifacts/bench-readiness/vcf-admixture-ready/render.lock";
