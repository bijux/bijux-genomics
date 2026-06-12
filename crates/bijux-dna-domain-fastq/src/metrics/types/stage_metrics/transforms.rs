use serde::{Deserialize, Serialize};

use super::super::common::{FastqDeltaMetricsV1, RetentionReportMetricV1};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqMergeMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub reads_r1: u64,
    #[serde(default)]
    pub reads_r2: u64,
    pub reads_merged: u64,
    #[serde(default)]
    pub reads_unmerged: u64,
    #[serde(default)]
    pub reads_discarded: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    #[serde(default)]
    pub input_pair_count: u64,
    #[serde(default)]
    pub merged_pair_count: u64,
    #[serde(default)]
    pub unmerged_pair_count: u64,
    #[serde(default)]
    pub discarded_pair_count: u64,
    pub merge_rate: f64,
    pub merge_q_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqCorrectMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub reads_corrected: u64,
    pub reads_uncorrected: u64,
    pub bases_corrected: u64,
    pub bases_uncorrected: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqUmiMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    #[serde(default)]
    pub umi_pattern: String,
    #[serde(default)]
    pub tag_header_format: String,
    #[serde(default)]
    pub reads_with_umi: u64,
    #[serde(default)]
    pub extracted_umi_count: u64,
    #[serde(default)]
    pub invalid_umi_count: u64,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub delta_metrics: FastqDeltaMetricsV1,
    pub retention: RetentionReportMetricV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqPreprocessMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub delta_metrics: FastqDeltaMetricsV1,
    pub retention: RetentionReportMetricV1,
}
