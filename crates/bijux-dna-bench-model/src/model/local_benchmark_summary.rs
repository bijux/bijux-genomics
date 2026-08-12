//! Persisted local benchmark summary contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{
    BenchmarkLocalStageReadinessKind, BenchmarkStageResultStatus, BENCHMARK_LOCAL_SUMMARY_SCHEMA_V1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkLocalReadinessStatus {
    Ready,
    MissingManifest,
    MissingOutputs,
    MissingManifestAndOutputs,
    Failed,
}

impl BenchmarkLocalReadinessStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::MissingManifest => "missing_manifest",
            Self::MissingOutputs => "missing_outputs",
            Self::MissingManifestAndOutputs => "missing_manifest_and_outputs",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSummaryStage {
    pub stage_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
    pub tool_id: String,
    pub readiness_status: BenchmarkLocalReadinessStatus,
    pub manifest_exists: bool,
    pub declared_output_count: usize,
    pub present_output_count: usize,
    pub missing_output_count: usize,
    pub runtime_seconds: String,
    pub runtime_status: BenchmarkStageResultStatus,
    pub memory_mb: String,
    pub output_metric: String,
    pub failure_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalSummaryReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub fake_run_root: String,
    pub report_output_path: String,
    pub markdown_output_path: String,
    pub source_stage_command_manifest_path: String,
    pub manifest_completion_report_path: String,
    pub output_completion_report_path: String,
    pub runtime_metrics_report_path: String,
    pub tool_comparison_template_path: String,
    pub stage_count: usize,
    pub ready_stage_count: usize,
    pub incomplete_stage_count: usize,
    pub failed_stage_count: usize,
    pub stages: Vec<BenchmarkLocalSummaryStage>,
}

#[must_use]
pub fn benchmark_local_summary_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalSummaryReport)
}

#[must_use]
pub const fn derive_benchmark_local_readiness_status(
    manifest_exists: bool,
    missing_output_count: usize,
    runtime_status: BenchmarkStageResultStatus,
) -> BenchmarkLocalReadinessStatus {
    match (manifest_exists, missing_output_count > 0, runtime_status) {
        (false, true, _) => BenchmarkLocalReadinessStatus::MissingManifestAndOutputs,
        (false, false, _) => BenchmarkLocalReadinessStatus::MissingManifest,
        (true, true, _) => BenchmarkLocalReadinessStatus::MissingOutputs,
        (true, false, BenchmarkStageResultStatus::Failed) => BenchmarkLocalReadinessStatus::Failed,
        (true, false, BenchmarkStageResultStatus::Succeeded) => {
            BenchmarkLocalReadinessStatus::Ready
        }
    }
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_SUMMARY_SCHEMA_V1,
    })
}
