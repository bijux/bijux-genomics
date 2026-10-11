//! Persisted local HPC execution-resolution contracts.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BENCHMARK_LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcExecutionResolverRow {
    pub tool_id: String,
    pub lookup_tool_id: String,
    pub execution_mode: String,
    pub execution_mode_summary: String,
    pub domains: Vec<String>,
    pub selected_stage_ids: Vec<String>,
    pub selected_job_count: usize,
    pub selected_job_ids: Vec<String>,
    pub selected_result_ids: Vec<String>,
    pub selected_pipeline_ids: Vec<String>,
    pub selected_node_ids: Vec<String>,
    pub install_kind: String,
    pub resolution_kind: String,
    pub resolution_target: String,
    pub command_entrypoint: Option<String>,
    pub runtime_probe_paths: Vec<String>,
    pub registry_paths: Vec<String>,
    pub unavailable_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcExecutionResolverReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub row_count: usize,
    pub resolution_counts: BTreeMap<String, usize>,
    pub unclassified_tool_count: usize,
    pub rows: Vec<BenchmarkLocalHpcExecutionResolverRow>,
}

#[must_use]
pub fn benchmark_local_hpc_execution_resolver_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcExecutionResolverReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BENCHMARK_LOCAL_HPC_EXECUTION_RESOLVER_SCHEMA_V1})
}
