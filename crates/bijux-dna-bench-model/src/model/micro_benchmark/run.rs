//! Owner: micro-benchmark persisted manifest and observation wire contracts.
use schemars::{json_schema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

pub const MICRO_BENCHMARK_RUN_SCHEMA_VERSION: &str = "bijux.bench.local_micro_benchmark_run.v1";
pub const MICRO_BENCHMARK_RUN_COMMAND: &str = "bijux-dna bench run-micro";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MicroBenchmarkResultKind {
    Stage,
    PipelineBridge,
    FamilyRepresentative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MicroBenchmarkExecutionStatus {
    Succeeded,
    ContainerNeeded,
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkComponentReport {
    pub component_id: String,
    pub report_path: String,
    pub row_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkResultRow {
    pub execution_id: String,
    pub component_id: String,
    pub result_kind: MicroBenchmarkResultKind,
    pub domain: String,
    pub bridge_source_domain: Option<String>,
    pub bridge_target_domain: Option<String>,
    pub stage_id: String,
    pub tool_id: String,
    pub status: MicroBenchmarkExecutionStatus,
    pub reason: String,
    pub command: Option<String>,
    pub source_report_path: String,
    pub evidence_path: Option<String>,
    pub stage_result_manifest_path: Option<String>,
    pub normalized_metric_count: usize,
    pub output_count: usize,
    pub log_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkOutputRow {
    pub execution_id: String,
    pub component_id: String,
    pub artifact_id: String,
    pub role: String,
    pub path: String,
    pub exists: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkLogRow {
    pub execution_id: String,
    pub component_id: String,
    pub role: String,
    pub path: String,
    pub exists: bool,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkNormalizedMetricRow {
    pub execution_id: String,
    pub component_id: String,
    pub metric_id: String,
    pub value: Value,
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkRunManifest {
    #[serde(deserialize_with = "deserialize_version")]
    #[schemars(schema_with = "version_schema")]
    pub schema_version: String,
    pub manifest_path: String,
    pub run_root: String,
    pub run_id: String,
    pub repo_revision: String,
    pub worktree_dirty: bool,
    pub created_at_unix: u64,
    pub command: String,
    pub component_reports: Vec<MicroBenchmarkComponentReport>,
    pub result_rows_path: String,
    pub output_rows_path: String,
    pub log_rows_path: String,
    pub normalized_metrics_path: String,
    pub result_row_count: usize,
    pub output_row_count: usize,
    pub log_row_count: usize,
    pub normalized_metric_row_count: usize,
    pub passes_behavior_test: bool,
}

fn deserialize_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != MICRO_BENCHMARK_RUN_SCHEMA_VERSION {
        return Err(serde::de::Error::custom("unsupported micro-benchmark run schema"));
    }
    Ok(version)
}
fn version_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":MICRO_BENCHMARK_RUN_SCHEMA_VERSION})
}
