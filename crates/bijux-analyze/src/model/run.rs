//! Owner: bijux-analyze
//! Typed run-level records.

use crate::model::JsonBlob;

#[derive(Debug, Clone)]
pub struct StageRecord {
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub params_hash: String,
    pub input_hash: String,
    pub metrics: JsonBlob,
}

#[derive(Debug, Clone)]
pub struct ToolRecord {
    pub tool_id: String,
    pub tool_version: String,
    pub records: Vec<StageRecord>,
}

#[derive(Debug, Clone)]
pub struct MetricEnvelope {
    pub metric_id: String,
    pub value: f64,
}

#[derive(Debug, Clone)]
pub struct RunSummary {
    pub run_id: String,
    pub stages: Vec<StageRecord>,
    pub reports: JsonBlob,
}
