use serde::{Deserialize, Serialize};

use super::common::{FastqDeltaMetricsV1, RetentionReportMetricV1};

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
    pub merge_rate: f64,
    pub merge_q_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqQcPostMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    #[serde(default)]
    pub mean_q: Option<f64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub delta_metrics: FastqDeltaMetricsV1,
    pub retention: RetentionReportMetricV1,
    #[serde(default)]
    pub contamination_rate: Option<f64>,
    #[serde(default)]
    pub aggregation_engine: Option<String>,
    #[serde(default)]
    pub aggregation_scope: Option<String>,
    #[serde(default)]
    pub adapter_content_max: Option<f64>,
    #[serde(default)]
    pub adapter_content_mean: Option<f64>,
    #[serde(default)]
    pub duplication_rate: Option<f64>,
    #[serde(default)]
    pub n_rate: Option<f64>,
    #[serde(default)]
    pub kmer_warning_count: Option<u64>,
    #[serde(default)]
    pub overrepresented_sequence_count: Option<u64>,
    #[serde(default)]
    pub raw_fastqc_dir: Option<String>,
    #[serde(default)]
    pub trimmed_fastqc_dir: Option<String>,
    #[serde(default)]
    pub multiqc_report: Option<String>,
    #[serde(default)]
    pub multiqc_data: Option<String>,
    #[serde(default)]
    pub governed_qc_input_count: Option<u64>,
    #[serde(default)]
    pub governed_qc_contributor_stage_ids: Vec<String>,
    #[serde(default)]
    pub governed_qc_contributor_tool_ids: Vec<String>,
    #[serde(default)]
    pub governed_qc_lineage_hash: Option<String>,
    #[serde(default)]
    pub multiqc_sample_count: Option<u64>,
    #[serde(default)]
    pub multiqc_module_count: Option<u64>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqValidateMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub reads_total: u64,
    pub reads_valid: u64,
    pub reads_invalid: u64,
    pub mean_q: f64,
    #[serde(default)]
    pub validated_inputs: Option<u64>,
    #[serde(default)]
    pub validated_pairs: Option<u64>,
    #[serde(default)]
    pub pair_sync_checked: Option<bool>,
    #[serde(default)]
    pub pair_sync_pass: Option<bool>,
    #[serde(default)]
    pub pair_count_match: Option<bool>,
    #[serde(default)]
    pub strict_pass: Option<bool>,
    #[serde(default)]
    pub failure_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqDetectAdaptersMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q: f64,
    #[serde(default)]
    pub adapter_content_max: Option<f64>,
    #[serde(default)]
    pub adapter_content_mean: Option<f64>,
    #[serde(default)]
    pub duplication_rate: Option<f64>,
    #[serde(default)]
    pub n_rate: Option<f64>,
    #[serde(default)]
    pub kmer_warning_count: Option<u64>,
    #[serde(default)]
    pub overrepresented_sequence_count: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqStatsNeutralMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    #[serde(default)]
    pub read_length_distribution: Vec<(u64, u64)>,
    #[serde(default)]
    pub gc_distribution: Vec<(u8, u64)>,
}
