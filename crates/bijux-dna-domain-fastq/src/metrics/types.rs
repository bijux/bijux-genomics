use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqDeltaMetricsV1 {
    pub read_retention: f64,
    pub base_retention: f64,
    pub mean_q_delta: f64,
    pub gc_delta: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetentionReportMetricV1 {
    pub value: f64,
    pub numerator_reads: u64,
    pub denominator_reads: u64,
    pub numerator_bases: u64,
    pub denominator_bases: u64,
    pub definition: String,
    pub stage_boundary: String,
    pub conditions: serde_json::Value,
}

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
    pub raw_fastqc_dir: Option<String>,
    #[serde(default)]
    pub trimmed_fastqc_dir: Option<String>,
    #[serde(default)]
    pub multiqc_report: Option<String>,
    #[serde(default)]
    pub multiqc_data: Option<String>,
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
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqQScoreSummaryV1 {
    pub mean_phred: f64,
    pub median_phred: f64,
    pub p10_phred: f64,
    pub p90_phred: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqQcSummaryMetricsV1 {
    pub reads: u64,
    pub bases_bp: u64,
    pub mean_read_length_bp: f64,
    pub qscore: FastqQScoreSummaryV1,
    #[serde(default)]
    pub duplication_estimate_pct: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqScanMetricsV1 {
    pub schema_version: String,
    pub summary: FastqQcSummaryMetricsV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeqfuMetricsV1 {
    pub schema_version: String,
    pub summary: FastqQcSummaryMetricsV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaxonomyRecordV1 {
    pub taxon_id: u64,
    pub taxon_name: String,
    pub rank: String,
    pub read_count: u64,
    #[serde(default)]
    pub fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClassificationDbProvenanceV1 {
    pub db_name: String,
    pub db_version: String,
    pub db_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KrakenUniqRecordV1 {
    pub taxonomy: TaxonomyRecordV1,
    pub unique_kmer_count: u64,
    #[serde(default)]
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrackenRecordV1 {
    pub taxonomy: TaxonomyRecordV1,
    pub estimated_reads: f64,
    #[serde(default)]
    pub estimated_fraction: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KrakenUniqClassificationMetricsV1 {
    pub schema_version: String,
    pub provenance: ClassificationDbProvenanceV1,
    pub taxonomy_table: Vec<KrakenUniqRecordV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BrackenClassificationMetricsV1 {
    pub schema_version: String,
    pub provenance: ClassificationDbProvenanceV1,
    pub taxonomy_table: Vec<BrackenRecordV1>,
}
