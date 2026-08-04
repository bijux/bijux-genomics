//! Owner: bijux-dna-bench-model
//! Canonical benchmark lifecycle contracts.
//! Owns typed attempt and cycle status payloads used by crate-backed benchmark operators.
//! Must not perform IO or depend on workflow orchestration.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{BENCHMARK_ATTEMPT_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V1};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkLifecycleStatus {
    Planned,
    Ready,
    Submitted,
    Running,
    Collecting,
    Passed,
    Failed,
    Superseded,
    BlockedExternal,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkJobReference {
    pub scheduler: String,
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAttemptRecord {
    #[schemars(schema_with = "benchmark_attempt_schema_version")]
    pub schema_version: String,
    pub campaign_id: String,
    pub run_id: String,
    pub attempt_id: String,
    pub stage_id: String,
    pub tool_id: String,
    pub status: BenchmarkLifecycleStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub jobs: Vec<BenchmarkJobReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_attempt_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCycleStatus {
    #[schemars(schema_with = "benchmark_cycle_status_schema_version")]
    pub schema_version: String,
    pub campaign_id: String,
    pub generated_at: String,
    pub attempts: Vec<BenchmarkAttemptRecord>,
}

impl BenchmarkAttemptRecord {
    #[must_use]
    pub fn v1(
        campaign_id: String,
        run_id: String,
        attempt_id: String,
        stage_id: String,
        tool_id: String,
        status: BenchmarkLifecycleStatus,
    ) -> Self {
        Self {
            schema_version: BENCHMARK_ATTEMPT_SCHEMA_V1.to_string(),
            campaign_id,
            run_id,
            attempt_id,
            stage_id,
            tool_id,
            status,
            jobs: Vec::new(),
            evidence_paths: Vec::new(),
            supersedes_attempt_id: None,
            blocked_reason: None,
        }
    }
}

impl BenchmarkCycleStatus {
    #[must_use]
    pub fn v1(
        campaign_id: String,
        generated_at: String,
        attempts: Vec<BenchmarkAttemptRecord>,
    ) -> Self {
        Self {
            schema_version: BENCHMARK_CYCLE_STATUS_SCHEMA_V1.to_string(),
            campaign_id,
            generated_at,
            attempts,
        }
    }
}

fn benchmark_attempt_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_ATTEMPT_SCHEMA_V1,
    })
}

fn benchmark_cycle_status_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
    })
}
