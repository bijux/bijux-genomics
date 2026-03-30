use serde::{Deserialize, Serialize};

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
