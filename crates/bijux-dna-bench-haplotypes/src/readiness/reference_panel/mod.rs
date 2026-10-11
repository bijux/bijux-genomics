//! Stage readiness separates scientific evidence, projection and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfPrepareReferencePanelReadyReport, VcfPrepareReferencePanelReadyRow};
pub use service::render_vcf_prepare_reference_panel_ready;
pub const DEFAULT_VCF_PREPARE_REFERENCE_PANEL_READY_PATH: &str =
    "artifacts/benchmark-smoke/readiness/vcf/prepare-reference-panel-ready.json";
pub(super) const VCF_PREPARE_REFERENCE_PANEL_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.vcf_prepare_reference_panel_ready.v1";
pub(super) const VCF_PREPARE_REFERENCE_PANEL_STAGE_ID: &str = "vcf.prepare_reference_panel";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 5] = [
    "input_variants",
    "output_variants",
    "sample_count",
    "duplicate_sites_removed",
    "normalization_status",
];
pub(super) const REQUIRED_OUTPUT_VARIANT_COUNT_MINIMUM: u64 = 1;
pub(super) const REQUIRED_NORMALIZATION_STATUS: &str = "sorted_indexed_deduplicated";
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
