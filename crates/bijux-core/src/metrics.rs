use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::ToolConstraints;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricSet<T> {
    pub metrics_schema: String,
    pub version: i32,
    pub metrics: T,
}

impl<T> MetricSet<T> {
    #[must_use]
    pub fn new(metrics_schema: String, version: i32, metrics: T) -> Self {
        Self {
            metrics_schema,
            version,
            metrics,
        }
    }
}

pub type MetricEnvelope<T> = MetricSet<T>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolInvocationV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub runner: String,
    pub platform: String,
    pub parameters_json: serde_json::Value,
    #[serde(default)]
    pub adapter_bank: Option<AdapterBankProvenanceV1>,
    pub resources: ToolConstraints,
    pub environment: BTreeMap<String, String>,
    pub input_hashes: Vec<String>,
    pub output_hashes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterBankProvenanceV1 {
    pub bank_id: String,
    pub bank_version: String,
    pub bank_hash: String,
    pub presets_hash: String,
    pub preset: String,
    pub enable_adapters: Vec<String>,
    pub disable_adapters: Vec<String>,
}

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
    pub retention: f64,
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub condition: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqTrimMetricsV1 {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
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
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub delta_metrics: FastqDeltaMetricsV1,
    pub retention: RetentionReportMetricV1,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqMergeMetricsV1 {
    pub reads_r1: u64,
    pub reads_r2: u64,
    pub reads_merged: u64,
    pub reads_unmerged: u64,
    pub merge_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqValidateMetricsV1 {
    pub reads_total: u64,
    pub reads_valid: u64,
    pub reads_invalid: u64,
    pub mean_q: f64,
}
