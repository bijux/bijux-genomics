use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastpToolMetricsV1 {
    pub schema_version: String,
    pub passed_filter_reads: u64,
    pub low_quality_reads: u64,
    pub too_many_n_reads: u64,
    pub too_short_reads: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterRemovalToolMetricsV1 {
    pub schema_version: String,
    pub pairs_processed: u64,
    pub pairs_merged: u64,
    pub merge_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SeqkitToolMetricsV1 {
    pub schema_version: String,
    pub reads: u64,
    pub bases: u64,
    pub mean_q: Option<f64>,
    pub gc_percent: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SamtoolsFlagstatMetricsV1 {
    pub schema_version: String,
    pub total_reads: u64,
    pub mapped_reads: u64,
    pub mapped_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqcToolMetricsV1 {
    pub schema_version: String,
    pub total_sequences: u64,
    pub gc_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MultiqcToolMetricsV1 {
    pub schema_version: String,
    pub sample_count: u64,
    pub module_count: u64,
}
