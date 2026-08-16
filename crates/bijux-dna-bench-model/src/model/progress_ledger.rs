//! Owner: bijux-dna-bench-model
//! Persisted finalization progress-ledger contract.
//! Keeps task state, execution-unit state, attempts, checks, and external waits distinct.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_PROGRESS_LEDGER_SCHEMA_V1;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkProgressLedger {
    #[schemars(schema_with = "progress_ledger_schema_version")]
    pub schema_version: String,
    #[schemars(schema_with = "cycle_id_schema")]
    pub cycle_id: String,
    pub source: BenchmarkProgressLedgerSource,
    pub frontier: BenchmarkProgressFrontier,
    pub tasks: Vec<BenchmarkProgressTask>,
    pub execution_units: Vec<BenchmarkProgressExecutionUnit>,
    pub attempts: Vec<BenchmarkProgressAttempt>,
    pub commits: Vec<BenchmarkProgressCommit>,
    pub checks: Vec<BenchmarkProgressCheck>,
    pub external_waits: Vec<BenchmarkProgressExternalWait>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkProgressLedgerSource {
    #[schemars(schema_with = "git_sha_schema")]
    pub local_head: String,
    #[schemars(schema_with = "git_sha_schema")]
    pub remote_head: String,
    #[schemars(schema_with = "sha256_schema")]
    pub catalog_sha256: String,
    #[schemars(schema_with = "sha256_schema")]
    pub benchmark_registry_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_clean: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote_clean: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkProgressFrontier {
    pub ready_task_ids: Vec<String>,
    pub ready_unit_ids: Vec<String>,
    pub submitted_unit_ids: Vec<String>,
    pub collectible_unit_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkProgressTaskStatus {
    Open,
    Ready,
    InProgress,
    WaitingExternal,
    BlockedExternal,
    Verified,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkProgressTask {
    pub task_id: String,
    pub status: BenchmarkProgressTaskStatus,
    pub next_action: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkProgressOwner {
    Containers,
    Benchmarks,
    Data,
    Ci,
    Publication,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkProgressExecutionUnitStatus {
    Planned,
    Ready,
    Submitted,
    Running,
    Collectible,
    Passed,
    Failed,
    Superseded,
    BlockedExternal,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkProgressExecutionUnit {
    pub unit_id: String,
    pub owner: BenchmarkProgressOwner,
    pub run_id: String,
    pub status: BenchmarkProgressExecutionUnitStatus,
    pub prerequisites: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attempt_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkProgressAttemptStatus {
    Prepared,
    Submitted,
    Pending,
    Running,
    Collectible,
    Passed,
    Failed,
    Superseded,
    BlockedExternal,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkProgressAttempt {
    pub attempt_id: String,
    pub run_id: String,
    #[schemars(schema_with = "git_sha_schema")]
    pub source_commit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slurm_job_id: Option<String>,
    pub status: BenchmarkProgressAttemptStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_root: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkProgressCommit {
    #[schemars(schema_with = "git_sha_schema")]
    pub sha: String,
    pub intent: String,
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkProgressCheckStatus {
    Passed,
    Failed,
    Running,
    NotRun,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkProgressCheck {
    pub id: String,
    pub status: BenchmarkProgressCheckStatus,
    pub log: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkProgressExternalWait {
    pub id: String,
    pub reason: String,
    #[schemars(schema_with = "true_schema")]
    pub independent_work_continues: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_observation_command: Option<String>,
}

#[must_use]
pub fn benchmark_progress_ledger_schema() -> Schema {
    schemars::schema_for!(BenchmarkProgressLedger)
}

fn progress_ledger_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BENCHMARK_PROGRESS_LEDGER_SCHEMA_V1})
}

fn cycle_id_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "minLength": 12})
}

fn git_sha_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "pattern": "^[0-9a-f]{40}$"})
}

fn sha256_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "pattern": "^[0-9a-f]{64}$"})
}

fn true_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "boolean", "const": true})
}
