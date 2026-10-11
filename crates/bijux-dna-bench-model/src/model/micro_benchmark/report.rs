//! Persisted local micro-benchmark reporting contracts.
use super::run::MicroBenchmarkExecutionStatus;
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub const MICRO_BENCHMARK_REPORT_SCHEMA_VERSION: &str =
    "bijux.bench.local_micro_benchmark_report.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkCompleteRow {
    pub execution_id: String,
    pub component_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub source_report_path: String,
    pub evidence_path: Option<String>,
    pub command: Option<String>,
    pub reason: String,
    pub normalized_metric_count: usize,
    pub output_count: usize,
    pub log_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkFailedRow {
    pub row_id: String,
    pub source_surface: String,
    pub component_id: Option<String>,
    pub execution_id: Option<String>,
    pub domain: Option<String>,
    pub stage_id: Option<String>,
    pub tool_id: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkMissingRow {
    pub component_id: String,
    pub execution_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub source_report_path: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkUnavailableRow {
    pub execution_id: String,
    pub component_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub execution_status: MicroBenchmarkExecutionStatus,
    pub source_report_path: String,
    pub command: Option<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkInsufficientDataRow {
    pub execution_id: String,
    pub component_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub source_report_path: String,
    pub evidence_path: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkRuntimeRow {
    pub execution_id: String,
    pub component_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub execution_status: MicroBenchmarkExecutionStatus,
    pub elapsed_seconds: Option<f64>,
    pub runtime_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkMemorySourceRow {
    pub execution_id: String,
    pub component_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub execution_status: MicroBenchmarkExecutionStatus,
    pub declared_memory_mb: Option<f64>,
    pub declared_cpu_threads: Option<u32>,
    pub observed_memory_mb: Option<f64>,
    pub observed_cpu_threads: Option<u32>,
    pub memory_source: String,
    pub resource_origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkScienceThresholdRow {
    pub domain: String,
    pub stage_id: String,
    pub metric_id: String,
    pub metric_name: String,
    pub unit: Option<String>,
    pub direction: String,
    pub tolerance_kind: String,
    pub tolerance_value: f64,
    pub pass_rule: String,
    pub insufficiency_behavior: String,
    pub required: bool,
    pub declaration_origin: String,
    pub covered_tool_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MicroBenchmarkReport {
    #[serde(deserialize_with = "deserialize_schema_version")]
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub markdown_output_path: String,
    pub json_output_path: String,
    pub micro_run_manifest_path: String,
    pub result_row_count: usize,
    pub complete_row_count: usize,
    pub failed_row_count: usize,
    pub missing_row_count: usize,
    pub unavailable_row_count: usize,
    pub insufficient_data_row_count: usize,
    pub runtime_row_count: usize,
    pub memory_source_row_count: usize,
    pub science_threshold_row_count: usize,
    pub passes_behavior_test: bool,
    pub execution_status_counts: BTreeMap<String, usize>,
    pub runtime_source_counts: BTreeMap<String, usize>,
    pub memory_source_counts: BTreeMap<String, usize>,
    pub complete_rows: Vec<MicroBenchmarkCompleteRow>,
    pub failed_rows: Vec<MicroBenchmarkFailedRow>,
    pub missing_rows: Vec<MicroBenchmarkMissingRow>,
    pub unavailable_rows: Vec<MicroBenchmarkUnavailableRow>,
    pub insufficient_data_rows: Vec<MicroBenchmarkInsufficientDataRow>,
    pub runtime_rows: Vec<MicroBenchmarkRuntimeRow>,
    pub memory_source_rows: Vec<MicroBenchmarkMemorySourceRow>,
    pub science_threshold_rows: Vec<MicroBenchmarkScienceThresholdRow>,
}

fn deserialize_schema_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != MICRO_BENCHMARK_REPORT_SCHEMA_VERSION {
        return Err(serde::de::Error::custom("unsupported micro benchmark report schema version"));
    }
    Ok(version)
}
fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":MICRO_BENCHMARK_REPORT_SCHEMA_VERSION})
}
