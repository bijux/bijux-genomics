use serde::{Deserialize, Serialize};

use super::super::common::{FastqDeltaMetricsV1, RetentionReportMetricV1};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqTrimMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub delta_metrics: FastqDeltaMetricsV1,
    #[serde(default)]
    pub paired_mode: Option<String>,
    #[serde(default)]
    pub adapter_policy: Option<String>,
    #[serde(default)]
    pub polyx_policy: Option<String>,
    #[serde(default)]
    pub n_policy: Option<String>,
    #[serde(default)]
    pub contaminant_policy: Option<String>,
    #[serde(default)]
    pub raw_backend_report_format: Option<String>,
    pub retention: RetentionReportMetricV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqFilterMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_dropped: u64,
    #[serde(default)]
    pub reads_removed_by_n: u64,
    #[serde(default)]
    pub reads_removed_by_entropy: u64,
    #[serde(default)]
    pub reads_removed_low_complexity: u64,
    #[serde(default)]
    pub reads_removed_by_kmer: u64,
    #[serde(default)]
    pub reads_removed_contaminant_kmer: u64,
    #[serde(default)]
    pub reads_removed_by_length: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub delta_metrics: FastqDeltaMetricsV1,
    pub retention: RetentionReportMetricV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqDeduplicateMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_removed_duplicates: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    #[serde(default)]
    pub paired_mode: Option<String>,
    #[serde(default)]
    pub dedup_mode: Option<String>,
    #[serde(default)]
    pub keep_order: Option<bool>,
    #[serde(default)]
    pub pair_count_match: Option<bool>,
    #[serde(default)]
    pub duplicate_class_count: Option<u64>,
    #[serde(default)]
    pub duplicate_provenance_json: Option<String>,
    #[serde(default)]
    pub raw_backend_report_format: Option<String>,
    pub delta_metrics: FastqDeltaMetricsV1,
    pub retention: RetentionReportMetricV1,
}
