//! Owner: bijux-dna-analyze
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FactsSummary {
    pub runs: usize,
    pub stages: usize,
    pub total_runtime_s: f64,
    pub avg_runtime_s: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunSummaryV1 {
    pub schema_version: String,
    pub facts_path: Option<String>,
    pub report_path: Option<String>,
    pub telemetry_path: Option<String>,
    pub final_outputs: Vec<String>,
    pub runs: usize,
    pub stages: usize,
    pub total_runtime_s: f64,
    pub avg_runtime_s: f64,
    pub stage_rows: Vec<RunSummaryStageRow>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunSummaryStageRow {
    pub run_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub tool_version: String,
    pub image_digest: Option<String>,
    pub params_hash: String,
    pub input_hash: String,
    pub bank_hashes: JsonBlob,
    pub runtime_s: f64,
    pub memory_mb: f64,
    pub exit_code: i32,
    pub reports: JsonBlob,
    pub deltas: RunSummaryDeltas,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RunSummaryDeltas {
    pub reads_in: Option<u64>,
    pub reads_out: Option<u64>,
    pub bases_in: Option<u64>,
    pub bases_out: Option<u64>,
    pub pairs_in: Option<u64>,
    pub pairs_out: Option<u64>,
}

pub(crate) fn stable_sort_records<T>(
    rows: &mut [T],
    key: impl Fn(&T) -> (&str, &str, &str, &str, &str),
) {
    rows.sort_by(|a, b| key(a).cmp(&key(b)));
}
