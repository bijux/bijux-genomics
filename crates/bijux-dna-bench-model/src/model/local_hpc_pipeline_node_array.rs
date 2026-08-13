//! Persisted local HPC pipeline-node array contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{BenchmarkLocalHpcArrayResources, BENCHMARK_LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_V1};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcPipelineNodeArrayExpectedOutput {
    pub output_id: String,
    pub output_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcPipelineNodeArrayDependency {
    pub node_id: String,
    pub job_id_local: String,
    pub array_index: usize,
    pub upstream_result_ids: Vec<String>,
    pub expected_outputs: Vec<BenchmarkLocalHpcPipelineNodeArrayExpectedOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcPipelineNodeArrayRow {
    pub array_index: usize,
    pub job_id_local: String,
    pub pipeline_id: String,
    pub node_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub dependency_job_ids: Vec<String>,
    pub upstream_result_ids: Vec<String>,
    pub dependencies: Vec<BenchmarkLocalHpcPipelineNodeArrayDependency>,
    pub expected_outputs: Vec<BenchmarkLocalHpcPipelineNodeArrayExpectedOutput>,
    pub script_path: String,
    pub pipeline_node_argv: Vec<String>,
    pub pipeline_node_command: String,
    pub scratch_root: String,
    pub input_links_root: String,
    pub output_root: String,
    pub log_root: String,
    pub stdout_path: String,
    pub stderr_path: String,
    pub resources: BenchmarkLocalHpcArrayResources,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcPipelineNodeArrayReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub script_path: String,
    pub manifest_path: String,
    pub wrapper_log_root: String,
    pub array_spec: String,
    pub pipeline_count: usize,
    pub pipeline_job_count: usize,
    pub dependency_count: usize,
    pub resource_envelope: BenchmarkLocalHpcArrayResources,
    pub rows: Vec<BenchmarkLocalHpcPipelineNodeArrayRow>,
}

#[must_use]
pub fn benchmark_local_hpc_pipeline_node_array_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcPipelineNodeArrayReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string","const":BENCHMARK_LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_V1})
}
