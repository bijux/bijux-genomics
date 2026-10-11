//! Persisted local HPC submission-readiness contracts.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::BENCHMARK_LOCAL_HPC_SUBMISSION_READINESS_SCHEMA_V1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcSubmissionReadinessGoalCheck {
    pub goal_id: u32,
    pub category: String,
    pub surface: String,
    pub output_path: Option<String>,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkLocalHpcSubmissionReadinessReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub checked_goal_count: usize,
    pub passed_goal_count: usize,
    pub failed_goal_count: usize,
    pub failing_goal_ids: Vec<u32>,
    pub ok: bool,
    pub checks: Vec<BenchmarkLocalHpcSubmissionReadinessGoalCheck>,
}

#[must_use]
pub fn benchmark_local_hpc_submission_readiness_schema() -> Schema {
    schemars::schema_for!(BenchmarkLocalHpcSubmissionReadinessReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_LOCAL_HPC_SUBMISSION_READINESS_SCHEMA_V1,
    })
}
