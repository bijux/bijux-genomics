//! Persisted local benchmark stage-completion contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{
    BenchmarkLocalStageReadinessKind, BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageManifestCompletionEntry {
    pub stage_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
    pub tool_id: String,
    pub stage_manifest_path: String,
    pub manifest_exists: bool,
    pub declared_output_count: usize,
    pub present_output_count: usize,
    pub complete: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageManifestCompletionReport {
    #[schemars(schema_with = "manifest_schema_version")]
    pub schema_version: String,
    pub fake_run_root: String,
    pub report_output_path: String,
    pub source_stage_command_manifest_path: String,
    pub stage_count: usize,
    pub complete_stage_count: usize,
    pub incomplete_stage_count: usize,
    pub complete: bool,
    pub stages: Vec<BenchmarkLocalStageManifestCompletionEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageMissingOutputEntry {
    pub artifact_id: String,
    pub declared_path: String,
    pub expected_fake_run_path: String,
    pub role: String,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageOutputCompletionEntry {
    pub stage_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
    pub tool_id: String,
    pub declared_output_count: usize,
    pub present_output_count: usize,
    pub missing_output_count: usize,
    pub complete: bool,
    pub missing_outputs: Vec<BenchmarkLocalStageMissingOutputEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageOutputCompletionReport {
    #[schemars(schema_with = "output_schema_version")]
    pub schema_version: String,
    pub fake_run_root: String,
    pub report_output_path: String,
    pub source_stage_command_manifest_path: String,
    pub stage_count: usize,
    pub complete_stage_count: usize,
    pub incomplete_stage_count: usize,
    pub complete: bool,
    pub stages: Vec<BenchmarkLocalStageOutputCompletionEntry>,
}

#[must_use]
pub fn benchmark_local_stage_manifest_completion_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalStageManifestCompletionReport)
}

#[must_use]
pub fn benchmark_local_stage_output_completion_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalStageOutputCompletionReport)
}

fn manifest_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1,
    })
}

fn output_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1,
    })
}
