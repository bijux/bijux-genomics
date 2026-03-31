use serde::{Deserialize, Serialize};

use super::super::common::{FastqDeltaMetricsV1, RetentionReportMetricV1};

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
