//! Persisted local HPC scratch-layout contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcScratchInputLink {
    pub artifact_id: Option<String>,
    pub artifact_role: Option<String>,
    pub source_kind: String,
    pub source_path: String,
    pub staged_path: String,
    pub link_path: String,
    pub checksum_sha256: String,
    pub size_bytes: u64,
    pub member_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcScratchJobResources {
    pub cpus_per_task: u32,
    pub memory_mb: u32,
    pub time_limit: String,
    pub scratch_gb: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchmarkLocalHpcScratchCleanupPolicy {
    pub policy_id: String,
    pub remove_scratch_after_successful_sync: bool,
    pub retain_scratch_on_failure: bool,
    pub preserve_output_root: bool,
    pub preserve_log_root: bool,
    pub preserve_staged_inputs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcScratchLayoutJob {
    pub job_id_local: String,
    pub job_kind: String,
    pub result_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub scratch_root: String,
    pub input_links_root: String,
    pub output_root: String,
    pub log_root: String,
    pub stdout_path: String,
    pub stderr_path: String,
    pub resources: BenchmarkLocalHpcScratchJobResources,
    pub cleanup_policy: BenchmarkLocalHpcScratchCleanupPolicy,
    pub input_links: Vec<BenchmarkLocalHpcScratchInputLink>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcScratchLayout {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub staging_root: String,
    pub scratch_root: String,
    pub selected_job_count: usize,
    pub benchmark_job_count: usize,
    pub essential_pipeline_job_count: usize,
    pub input_link_count: usize,
    pub jobs: Vec<BenchmarkLocalHpcScratchLayoutJob>,
}

#[must_use]
pub fn benchmark_local_hpc_scratch_layout_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcScratchLayout)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_HPC_SCRATCH_LAYOUT_SCHEMA_V1,
    })
}
