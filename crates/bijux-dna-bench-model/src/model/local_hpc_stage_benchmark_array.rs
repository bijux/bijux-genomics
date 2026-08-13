//! Persisted local HPC stage-benchmark array contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{BenchmarkLocalHpcArrayResources, BENCHMARK_LOCAL_HPC_STAGE_BENCHMARK_ARRAY_SCHEMA_V1};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcStageBenchmarkArrayRow {
    pub array_index: usize,
    pub job_id_local: String,
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub execution_mode: String,
    pub execution_mode_summary: String,
    pub resolution_kind: String,
    pub resolution_target: String,
    pub unavailable_reason: Option<String>,
    pub script_path: String,
    pub benchmark_result_argv: Vec<String>,
    pub benchmark_result_command: String,
    pub scratch_root: String,
    pub input_links_root: String,
    pub output_root: String,
    pub log_root: String,
    pub stdout_path: String,
    pub stderr_path: String,
    pub output_paths: Vec<String>,
    pub resources: BenchmarkLocalHpcArrayResources,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcStageBenchmarkArrayReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub script_path: String,
    pub manifest_path: String,
    pub wrapper_log_root: String,
    pub array_spec: String,
    pub benchmark_job_count: usize,
    pub resource_envelope: BenchmarkLocalHpcArrayResources,
    pub rows: Vec<BenchmarkLocalHpcStageBenchmarkArrayRow>,
}

#[must_use]
pub fn benchmark_local_hpc_stage_benchmark_array_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcStageBenchmarkArrayReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string","const":BENCHMARK_LOCAL_HPC_STAGE_BENCHMARK_ARRAY_SCHEMA_V1})
}
