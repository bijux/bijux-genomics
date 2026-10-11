//! Persisted BAM duplicates reports and schema identities.

use super::alignment::BamFlagstatCountsV1;
use super::coverage::BamInsertSizeHistogramBinV1;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const BAM_DUPLICATE_INSERT_TRUTH_SUMMARY_SCHEMA_VERSION: &str =
    "bijux.bam.duplicate_insert_truth.v1";

pub const BAM_MARKDUP_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.markdup.v1";

pub const BAM_DUPLICATION_METRICS_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.duplication_metrics.v1";

pub const BAM_COMPLEXITY_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.complexity.v1";

pub const BAM_DUPLICATE_POLICY_SCHEMA_VERSION: &str = "bijux.bam.duplicate_policy.v1";

pub const BAM_DUPLICATE_COMPARISON_SCHEMA_VERSION: &str = "bijux.bam.duplicate_comparison.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamDuplicateFamilyHistogramBinV1 {
    pub family_size: u64,
    pub family_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamDuplicateInsertTruthSummaryV1 {
    pub schema_version: String,
    pub total_reads: u64,
    pub mapped_reads: u64,
    pub unmapped_reads: u64,
    pub examined_reads: u64,
    pub duplicate_reads: u64,
    pub duplicate_pairs: u64,
    pub duplicate_family_histogram: Vec<BamDuplicateFamilyHistogramBinV1>,
    pub pair_count: u64,
    pub insert_size_histogram: Vec<BamInsertSizeHistogramBinV1>,
    #[serde(default)]
    pub median_insert_size: Option<f64>,
    #[serde(default)]
    pub mean_insert_size: Option<f64>,
    #[serde(default)]
    pub min_insert_size: Option<u64>,
    #[serde(default)]
    pub max_insert_size: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamMarkdupSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub duplicate_action: String,
    #[serde(default)]
    pub optical_duplicates: Option<String>,
    #[serde(default)]
    pub umi_policy: Option<String>,
    pub input_bam: PathBuf,
    pub output_bam: PathBuf,
    pub flagstat_before: BamFlagstatCountsV1,
    pub flagstat_after: BamFlagstatCountsV1,
    pub input_reads: u64,
    pub output_reads: u64,
    pub removed_reads: u64,
    #[serde(default)]
    pub duplicate_reads_before: Option<u64>,
    #[serde(default)]
    pub duplicate_reads_after: Option<u64>,
    #[serde(default)]
    pub duplicate_count: Option<u64>,
    #[serde(default)]
    pub newly_marked_reads: Option<u64>,
    #[serde(default)]
    pub duplicate_reads_removed: Option<u64>,
    #[serde(default)]
    pub duplicate_fraction_before: Option<f64>,
    #[serde(default)]
    pub duplicate_fraction_after: Option<f64>,
    #[serde(default)]
    pub duplicate_fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamDuplicationMetricsSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub method: String,
    pub input_bam: PathBuf,
    pub examined_reads: u64,
    pub duplicate_reads: u64,
    #[serde(default)]
    pub duplicate_count: Option<u64>,
    pub duplicate_fraction: f64,
    #[serde(default)]
    pub estimated_library_size: Option<u64>,
    #[serde(default)]
    pub insufficient_library_size_reason: Option<String>,
    #[serde(default)]
    pub optical_duplicates: Option<String>,
    #[serde(default)]
    pub umi_policy: Option<String>,
    #[serde(default)]
    pub duplicate_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamComplexitySummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub method: String,
    pub input_bam: PathBuf,
    pub observed_total_reads: u64,
    pub observed_unique_reads: u64,
    #[serde(default)]
    pub projected_unique_reads: Vec<(u64, u64)>,
    #[serde(default)]
    pub estimated_unique_reads: Option<u64>,
    #[serde(default)]
    pub estimated_library_size: Option<u64>,
    #[serde(default)]
    pub saturation_estimate: Option<f64>,
    pub min_reads: u64,
    #[serde(default)]
    pub insufficient_data_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamDuplicateMethodMetricsV1 {
    pub method: String,
    #[serde(default)]
    pub duplicate_reads: Option<u64>,
    #[serde(default)]
    pub duplicate_fraction: Option<f64>,
    #[serde(default)]
    pub optical_duplicates: Option<String>,
    #[serde(default)]
    pub duplicate_action: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamDuplicateComparisonV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub method_a: BamDuplicateMethodMetricsV1,
    pub method_b: BamDuplicateMethodMetricsV1,
    pub comparable: bool,
    #[serde(default)]
    pub duplicate_reads_delta: Option<i64>,
    #[serde(default)]
    pub duplicate_fraction_delta: Option<f64>,
    pub policy_explicit: bool,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BamDuplicatePolicyV1 {
    pub schema_version: String,
    pub stage_id: String,
    #[serde(default)]
    pub library_type: Option<String>,
    #[serde(default)]
    pub optical_duplicates: Option<String>,
    #[serde(default)]
    pub umi_policy: Option<String>,
    #[serde(default)]
    pub duplicate_action: Option<String>,
    pub policy_scope: String,
    #[serde(default)]
    pub library_semantics: Vec<String>,
    #[serde(default)]
    pub comparison_ready_with: Vec<String>,
}
