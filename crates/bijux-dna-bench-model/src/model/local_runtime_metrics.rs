//! Persisted local benchmark runtime-metrics contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{BenchmarkStageResultStatus, BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkLocalStageReadinessKind {
    DryRun,
    Smoke,
    DryOrSmoke,
}

impl BenchmarkLocalStageReadinessKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::DryRun => "dry_run",
            Self::Smoke => "smoke",
            Self::DryOrSmoke => "dry_or_smoke",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageRuntimeMetric {
    pub stage_id: String,
    pub readiness_kind: BenchmarkLocalStageReadinessKind,
    pub tool_id: String,
    pub manifest_path: String,
    pub runtime_mode: String,
    pub started_at: String,
    pub finished_at: String,
    pub elapsed_seconds: f64,
    pub exit_code: i32,
    pub status: BenchmarkStageResultStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalStageRuntimeMetricsReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub fake_run_root: String,
    pub report_output_path: String,
    pub source_stage_command_manifest_path: String,
    pub stage_count: usize,
    pub stages: Vec<BenchmarkLocalStageRuntimeMetric>,
}

#[must_use]
pub fn benchmark_local_runtime_metrics_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalStageRuntimeMetricsReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_RUNTIME_METRICS_SCHEMA_V1,
    })
}
