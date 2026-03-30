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
