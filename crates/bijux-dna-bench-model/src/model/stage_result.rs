//! Canonical persisted result contract for one benchmark stage execution.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{
    BENCHMARK_STAGE_RESULT_SCHEMA_V2, BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStageResultStatus {
    Succeeded,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageResultTool {
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageResultCommand {
    pub rendered: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageResultRuntime {
    pub mode: String,
    pub status: BenchmarkStageResultStatus,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStageResultResourceMetricSource {
    Measured,
    Estimated,
    NotAvailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageResultResourceMetrics {
    pub source: BenchmarkStageResultResourceMetricSource,
    pub memory_mb: Option<f64>,
    pub cpu_threads: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageResultOutput {
    pub artifact_id: String,
    pub declared_path: String,
    pub realized_path: String,
    pub role: String,
    pub optional: bool,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageResultManifest {
    #[schemars(schema_with = "benchmark_stage_result_schema_version")]
    pub schema_version: String,
    pub stage_id: String,
    pub tool: BenchmarkStageResultTool,
    pub command: BenchmarkStageResultCommand,
    pub runtime: BenchmarkStageResultRuntime,
    pub resource_metrics: BenchmarkStageResultResourceMetrics,
    pub outputs: Vec<BenchmarkStageResultOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageResultValidationReport {
    #[schemars(schema_with = "benchmark_stage_result_validation_schema_version")]
    pub schema_version: String,
    pub manifest_path: String,
    pub stage_id: String,
    pub tool_id: String,
    pub output_count: usize,
    pub status: BenchmarkStageResultStatus,
    #[schemars(schema_with = "benchmark_stage_result_validation_valid")]
    pub valid: bool,
}

#[must_use]
pub fn benchmark_stage_result_schema() -> Schema {
    schemars::schema_for!(BenchmarkStageResultManifest)
}

#[must_use]
pub fn benchmark_stage_result_validation_schema() -> Schema {
    schemars::schema_for!(BenchmarkStageResultValidationReport)
}

fn benchmark_stage_result_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_STAGE_RESULT_SCHEMA_V2,
    })
}

fn benchmark_stage_result_validation_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1,
    })
}

fn benchmark_stage_result_validation_valid(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "boolean",
        "const": true,
    })
}
