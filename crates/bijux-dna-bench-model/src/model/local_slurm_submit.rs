//! Persisted local Slurm submission contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{
    BenchmarkLocalStageArtifactEntry, BenchmarkLocalStageReadinessKind,
    BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalAllDomainSlurmSubmitManifest {
    #[schemars(schema_with = "all_domain_schema_version")]
    pub schema_version: String,
    pub root_path: String,
    pub manifest_path: String,
    pub run_id: String,
    pub job_count: usize,
    pub dependency_count: usize,
    pub benchmark_job_count: usize,
    pub essential_pipeline_job_count: usize,
    pub jobs: Vec<BenchmarkLocalAllDomainSlurmSubmitJob>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalAllDomainSlurmSubmitJob {
    pub job_id_local: String,
    pub domain: String,
    pub stage_id: String,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub result_id: Option<String>,
    pub script_path: String,
    pub stdout: String,
    pub stderr: String,
    pub outputs: Vec<String>,
    pub dependencies: Vec<String>,
    pub resources: BenchmarkLocalSlurmSubmitResources,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSlurmSubmitManifest {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub root_path: String,
    pub manifest_path: String,
    pub run_id: String,
    pub job_count: usize,
    pub dependency_count: usize,
    pub jobs: Vec<BenchmarkLocalSlurmSubmitJob>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSlurmSubmitJob {
    pub job_name: String,
    pub domain: String,
    pub stage_id: Option<String>,
    pub pipeline_id: Option<String>,
    pub corpus_id: Option<String>,
    pub sample_id: Option<String>,
    pub sample_ids: Vec<String>,
    pub tool_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
    pub resources: BenchmarkLocalSlurmSubmitResources,
    pub script_path: String,
    pub logs: BenchmarkLocalSlurmSubmitLogs,
    pub result_root: String,
    pub outputs: Vec<BenchmarkLocalStageArtifactEntry>,
    pub dependencies: Vec<String>,
    pub compatibility_kind: String,
    pub compatibility_note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSlurmSubmitResources {
    pub cpus_per_task: u32,
    pub memory_mb: u32,
    pub time_limit: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSlurmSubmitLogs {
    pub stdout_path: String,
    pub stderr_path: String,
}

#[must_use]
pub fn benchmark_local_slurm_submit_manifest_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalSlurmSubmitManifest)
}

#[must_use]
pub fn benchmark_local_all_domain_slurm_submit_manifest_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalAllDomainSlurmSubmitManifest)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
    })
}

fn all_domain_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
    })
}
