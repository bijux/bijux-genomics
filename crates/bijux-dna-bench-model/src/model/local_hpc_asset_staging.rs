//! Persisted local HPC asset-staging contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcAssetStagingInput {
    pub artifact_id: Option<String>,
    pub artifact_role: Option<String>,
    pub source_kind: String,
    pub source_path: String,
    pub staged_path: String,
    pub checksum_sha256: String,
    pub size_bytes: u64,
    pub member_count: usize,
    pub consumer_result_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcAssetStagingJob {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub staged_inputs: Vec<BenchmarkLocalHpcAssetStagingInput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcAssetStagingManifest {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub staging_root: String,
    pub selected_job_count: usize,
    pub staged_input_count: usize,
    pub unique_source_path_count: usize,
    pub jobs: Vec<BenchmarkLocalHpcAssetStagingJob>,
}

#[must_use]
pub fn benchmark_local_hpc_asset_staging_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcAssetStagingManifest)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1,
    })
}
