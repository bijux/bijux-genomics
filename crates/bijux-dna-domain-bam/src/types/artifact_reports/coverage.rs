//! Persisted BAM coverage reports and schema identities.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const BAM_GC_BIAS_TRUTH_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.gc_bias_truth.v1";

pub const BAM_COVERAGE_TRUTH_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.coverage_truth.v1";

pub const BAM_COVERAGE_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.coverage_summary.v1";

pub const BAM_INSERT_SIZE_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.insert_size.v1";

pub const BAM_GC_BIAS_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.gc_bias.v1";

pub const BAM_BIAS_MITIGATION_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.bias_mitigation_summary.v1";

pub const BAM_COVERAGE_REGIME_SCHEMA_VERSION: &str = "bijux.bam.coverage_regime.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamInsertSizeHistogramBinV1 {
    pub insert_size: u64,
    pub pair_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamGcBiasTruthSummaryV1 {
    pub schema_version: String,
    pub window_size: u32,
    pub total_clusters: u64,
    pub aligned_reads: u64,
    pub windows: u64,
    pub read_starts: u64,
    pub at_dropout: f64,
    pub gc_dropout: f64,
    pub gc_bias_score: f64,
    #[serde(default)]
    pub insufficient_reference_reason: Option<String>,
    pub gc_bins: Vec<BamGcBiasBinSummaryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamCoverageTruthSummaryV1 {
    pub schema_version: String,
    pub depth_thresholds: Vec<u32>,
    pub mean_depth: f64,
    pub breadth_1x: f64,
    pub covered_bases: u64,
    pub total_bases: u64,
    pub coverage_regime: String,
    pub coverage_family: String,
    pub region_summaries: Vec<BamCoverageRegionSummaryV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamInsertSizeSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_bam: PathBuf,
    pub report_present: bool,
    pub histogram_present: bool,
    #[serde(default)]
    pub median_insert_size: Option<f64>,
    #[serde(default)]
    pub mean_insert_size: Option<f64>,
    #[serde(default)]
    pub standard_deviation: Option<f64>,
    #[serde(default)]
    pub median_absolute_deviation: Option<f64>,
    #[serde(default)]
    pub min_insert_size: Option<u64>,
    #[serde(default)]
    pub max_insert_size: Option<u64>,
    pub read_pairs: u64,
    #[serde(default)]
    pub pair_orientation_fr_fraction: Option<f64>,
    #[serde(default)]
    pub insufficient_pairs_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamGcBiasBinSummaryV1 {
    pub gc_bin: u8,
    pub windows: u64,
    pub read_starts: u64,
    pub normalized_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamGcBiasSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_bam: PathBuf,
    pub reference_fasta: PathBuf,
    #[serde(default)]
    pub window_size: Option<u32>,
    pub report_present: bool,
    pub plot_present: bool,
    pub total_clusters: u64,
    pub aligned_reads: u64,
    pub windows: u64,
    pub read_starts: u64,
    pub at_dropout: f64,
    pub gc_dropout: f64,
    pub gc_bias_score: f64,
    #[serde(default)]
    pub insufficient_reference_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamBiasMitigationSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_bam: PathBuf,
    #[serde(default)]
    pub reference_fasta: Option<PathBuf>,
    pub method: String,
    pub metric_name: String,
    pub gc_bias_correction: bool,
    pub map_bias_correction: bool,
    pub report_present: bool,
    #[serde(default)]
    pub mitigation_actions: Vec<String>,
    #[serde(default)]
    pub consumed_metrics: Vec<String>,
    #[serde(default)]
    pub pre_mitigation_metric: Option<f64>,
    #[serde(default)]
    pub post_mitigation_metric: Option<f64>,
    #[serde(default)]
    pub metric_delta: Option<f64>,
    #[serde(default)]
    pub mitigation_projection_basis: Option<String>,
    #[serde(default)]
    pub insufficient_metric_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BamCoverageRegimeClassV1 {
    Unusable,
    Sparse,
    LowPass,
    TargetLike,
    WholeGenomeLike,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamCoverageRegimeV1 {
    pub schema_version: String,
    pub regime_id: String,
    pub advisory_label: String,
    pub enforced_label: String,
    pub regime_class: BamCoverageRegimeClassV1,
    pub mean_depth: f64,
    pub breadth_1x: f64,
    pub usable_for: Vec<String>,
    #[serde(default)]
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamCoverageSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub has_mosdepth_summary: bool,
    pub has_samtools_depth: bool,
    #[serde(default)]
    pub mean_depth: Option<f64>,
    #[serde(default)]
    pub coverage_regime: Option<String>,
    #[serde(default)]
    pub coverage_family: Option<String>,
    #[serde(default)]
    pub regime: Option<BamCoverageRegimeV1>,
    #[serde(default)]
    pub depth_thresholds: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamCoverageRegionSummaryV1 {
    pub region_id: String,
    pub contig: String,
    pub start: u64,
    pub end: u64,
    pub length: u64,
    pub mean_depth: f64,
    pub breadth_1x: f64,
    pub covered_bases: u64,
}
