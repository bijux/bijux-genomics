//! Stage readiness separates observation, surface policy and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfDamageFilterReadyReport, VcfDamageFilterReadyRow};
pub use service::render_vcf_damage_filter_ready;
pub const DEFAULT_VCF_DAMAGE_FILTER_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/damage-filter-ready.json";
pub(super) const VCF_DAMAGE_FILTER_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_damage_filter_ready.v1";
pub(super) const VCF_DAMAGE_FILTER_STAGE_ID: &str = "vcf.damage_filter";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 9] = [
    "input_variants",
    "removed_variants",
    "retained_variants",
    "low_quality_filtered_variants",
    "damage_ratio_filtered_variants",
    "terminal_damage_filtered_variants",
    "damage_context_rule",
    "terminal_context_count",
    "sample_count",
];
pub(super) const REQUIRED_DAMAGE_CONTEXT_RULE: &str =
    "remove_ct_ga_with_ratio_gt_0.35_or_terminal_signal_ge_0.50_or_pmd_lt_3.0";
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
