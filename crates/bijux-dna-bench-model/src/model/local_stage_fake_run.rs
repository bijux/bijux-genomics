//! Persisted local stage fake-run and fake-failure contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{
    BenchmarkLocalStageReadinessKind, BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_RECORD_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_FAKE_RUN_MANIFEST_SCHEMA_V1, BENCHMARK_STAGE_RESULT_SCHEMA_V2,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageFakeRunOutputEntry {
    pub artifact_id: String,
    pub declared_path: String,
    pub fake_run_path: String,
    pub role: String,
    pub optional: bool,
    pub exists: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageFakeRunResult {
    #[schemars(schema_with = "stage_result_schema_version")]
    pub schema_version: String,
    pub stage_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
    pub tool_id: String,
    pub command: String,
    pub stage_manifest_path: String,
    pub declared_output_count: usize,
    pub created_output_count: usize,
    pub outputs: Vec<BenchmarkLocalStageFakeRunOutputEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageFakeRunManifest {
    #[schemars(schema_with = "fake_run_manifest_schema_version")]
    pub schema_version: String,
    pub fake_run_root: String,
    pub source_stage_command_manifest_path: String,
    pub stage_count: usize,
    pub stages: Vec<BenchmarkLocalStageFakeRunResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageFakeFailureOutputEntry {
    pub artifact_id: String,
    pub declared_path: String,
    pub expected_fake_run_path: String,
    pub role: String,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageFakeFailureRecord {
    #[schemars(schema_with = "fake_failure_record_schema_version")]
    pub schema_version: String,
    pub stage_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
    pub tool_id: String,
    pub command: String,
    pub exit_code: i32,
    pub stderr_path: String,
    pub failure_record_path: String,
    pub failed_output_count: usize,
    pub failed_outputs: Vec<BenchmarkLocalStageFakeFailureOutputEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageFakeFailureManifest {
    #[schemars(schema_with = "fake_failure_manifest_schema_version")]
    pub schema_version: String,
    pub failure_root: String,
    pub source_stage_command_manifest_path: String,
    pub stage_count: usize,
    pub failures: Vec<BenchmarkLocalStageFakeFailureRecord>,
}

#[must_use]
pub fn benchmark_local_stage_fake_run_manifest_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalStageFakeRunManifest)
}

#[must_use]
pub fn benchmark_local_stage_fake_failure_record_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalStageFakeFailureRecord)
}

#[must_use]
pub fn benchmark_local_stage_fake_failure_manifest_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalStageFakeFailureManifest)
}

fn stage_result_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BENCHMARK_STAGE_RESULT_SCHEMA_V2})
}

fn fake_run_manifest_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_STAGE_FAKE_RUN_MANIFEST_SCHEMA_V1,
    })
}

fn fake_failure_record_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_RECORD_SCHEMA_V1,
    })
}

fn fake_failure_manifest_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_MANIFEST_SCHEMA_V1,
    })
}
