//! Stage readiness separates observation, surface policy and compatible publication.
mod contract;
mod coverage;
mod fields;
mod model;
mod projection;
mod service;
pub use model::{VcfQcReadyReport, VcfQcReadyRow};
pub use service::render_vcf_qc_ready;
pub const DEFAULT_VCF_QC_READY_PATH: &str = "artifacts/benchmark-smoke/readiness/vcf/qc-ready.json";
pub(super) const VCF_QC_READY_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_qc_ready.v1";
pub(super) const VCF_QC_STAGE_ID: &str = "vcf.qc";
pub(super) const REQUIRED_METRIC_NAMES: [&str; 9] = [
    "sample_missingness",
    "variant_missingness",
    "maf_summary",
    "heterozygosity",
    "hwe_summary",
    "excluded_samples",
    "excluded_variants",
    "sample_missingness_exclusion_threshold",
    "variant_missingness_exclusion_threshold",
];
pub(super) const REQUIRED_SAMPLE_MISSINGNESS_EXCLUSION_THRESHOLD: f64 = 0.5;
pub(super) const REQUIRED_VARIANT_MISSINGNESS_EXCLUSION_THRESHOLD: f64 = 0.5;
pub(super) const REQUIRED_MAF_OBSERVED_VARIANT_COUNT: u64 = 4;
pub(super) const REQUIRED_HETEROZYGOUS_CALL_COUNT: u64 = 4;
pub(super) const REQUIRED_HOMOZYGOUS_ALT_CALL_COUNT: u64 = 2;
pub(super) const REQUIRED_HET_HOM_RATIO: f64 = 2.0;
pub(super) const REQUIRED_HWE_TESTED_VARIANT_COUNT: u64 = 3;
pub(super) const REQUIRED_HWE_PVALUE_MEAN: f64 = 0.825_656;
pub(super) const REQUIRED_HWE_STATUS: &str = "computed_modern";
pub(super) const REQUIRED_EXCLUDED_SAMPLE_ID: &str = "qc_sparse";
pub(super) const REQUIRED_EXCLUDED_SAMPLE_MISSINGNESS: f64 = 0.75;
pub(super) const REQUIRED_EXCLUDED_VARIANT_ID: &str = "chr1:30:G:A";
pub(super) const REQUIRED_EXCLUDED_VARIANT_MISSINGNESS: f64 = 2.0 / 3.0;
pub(super) const COVERAGE_STATUS_COMPLETE: &str = "complete";
pub(super) const COVERAGE_STATUS_INCOMPLETE: &str = "incomplete";
pub(super) const NO_VALUE: &str = "none";
pub(super) const FLOAT_TOLERANCE: f64 = 1e-9;

mod science;
