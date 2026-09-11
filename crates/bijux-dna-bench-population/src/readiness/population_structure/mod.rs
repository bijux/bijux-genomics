//! Stage readiness separates observation, scientific policy and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod science;
mod service;
pub use model::{VcfPopulationStructureReadyReport, VcfPopulationStructureReadyRow};
pub use service::render_vcf_population_structure_ready;
pub const DEFAULT_VCF_POPULATION_STRUCTURE_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/population-structure-ready.json";
pub(super) const VCF_POPULATION_STRUCTURE_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_population_structure_ready.v1";
pub(super) const VCF_POPULATION_STRUCTURE_STAGE_ID: &str = "vcf.population_structure";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 6] = [
    "consumed_admixture",
    "consumed_pca",
    "distance_summary",
    "sample_groups",
    "status",
    "variant_count",
];
pub(super) const REQUIRED_STATUS: &str = "complete";
pub(super) const REQUIRED_SAMPLE_COUNT: u64 = 4;
pub(super) const REQUIRED_PAIR_COUNT: u64 = 6;
pub(super) const REQUIRED_WITHIN_POPULATION_PAIR_COUNT: u64 = 2;
pub(super) const REQUIRED_CROSS_POPULATION_PAIR_COUNT: u64 = 4;
pub(super) const REQUIRED_SELECTED_K: u64 = 2;
pub(super) const REQUIRED_SAMPLE_IDS: [&str; 4] = ["sample_a", "sample_b", "sample_c", "sample_d"];
pub(super) const GOVERNED_STAGE_CONTRACT_SMOKE_TOOL_ID: &str = "plink2";
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
pub(super) const VCF_POPULATION_STRUCTURE_READY_LOCK_PATH: &str =
    "artifacts/bench-readiness/vcf-population-structure-ready/render.lock";
