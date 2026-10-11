//! Owner: bijux-dna-bench-model
//! Canonical benchmark lifecycle contracts.
//! Owns typed attempt and cycle status payloads used by crate-backed benchmark operators.
//! Must not perform IO or depend on workflow orchestration.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{
    BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1, BENCHMARK_ATTEMPT_MIGRATION_LEDGER_SCHEMA_V1,
    BENCHMARK_ATTEMPT_MIGRATION_PLAN_SCHEMA_V1, BENCHMARK_ATTEMPT_SCHEMA_V1,
    BENCHMARK_ATTEMPT_SCHEMA_V2, BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
    BENCHMARK_CYCLE_STATUS_SCHEMA_V2,
};

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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkAttemptDisposition {
    Failed,
    Superseded,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkArchiveSourcePaths {
    #[schemars(schema_with = "study_uri_schema")]
    pub submission: String,
    #[schemars(schema_with = "study_uri_schema")]
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkArchiveFileRecord {
    #[schemars(schema_with = "archive_relative_path_schema")]
    pub path: String,
    pub size_bytes: u64,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAttemptArchive {
    #[schemars(schema_with = "benchmark_attempt_archive_schema_version")]
    pub schema_version: String,
    pub run_id: String,
    pub attempt_id: String,
    pub tool_id: String,
    pub slurm_job_id: String,
    pub disposition: BenchmarkAttemptDisposition,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub archived_at_utc: String,
    pub source_paths: BenchmarkArchiveSourcePaths,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub evidence_sha256: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub accounting_sha256: String,
    pub files: Vec<BenchmarkArchiveFileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAttemptMigrationEntry {
    pub run_id: String,
    pub attempt_id: String,
    pub tool_id: String,
    pub slurm_job_id: String,
    pub disposition: BenchmarkAttemptDisposition,
    #[schemars(schema_with = "study_uri_schema")]
    pub source: String,
    #[schemars(schema_with = "study_uri_schema")]
    pub target: String,
    pub files: Vec<BenchmarkArchiveFileRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAttemptMigrationExclusion {
    #[schemars(schema_with = "study_uri_schema")]
    pub source: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAttemptMigrationPlan {
    #[schemars(schema_with = "benchmark_attempt_migration_plan_schema_version")]
    pub schema_version: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub plan_id: String,
    pub created_at_utc: String,
    pub entries: Vec<BenchmarkAttemptMigrationEntry>,
    pub exclusions: Vec<BenchmarkAttemptMigrationExclusion>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkAttemptMigrationState {
    Planned,
    Applied,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAttemptMigrationLedgerEntry {
    pub source: String,
    pub target: String,
    pub state: BenchmarkAttemptMigrationState,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAttemptMigrationLedger {
    #[schemars(schema_with = "benchmark_attempt_migration_ledger_schema_version")]
    pub schema_version: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub plan_sha256: String,
    pub updated_at_utc: String,
    pub entries: Vec<BenchmarkAttemptMigrationLedgerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkJobReference {
    pub scheduler: String,
    pub job_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRunIdentity {
    pub run_id: String,
    pub domain: String,
    pub stage_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAttemptIdentity {
    pub campaign_id: String,
    pub run: BenchmarkRunIdentity,
    pub attempt_id: String,
    pub tool_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAttemptTransition {
    pub sequence: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_status: Option<BenchmarkLifecycleStatus>,
    pub to_status: BenchmarkLifecycleStatus,
    pub observed_at_utc: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub jobs: Vec<BenchmarkJobReference>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence_paths: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAttemptRecordV2 {
    #[schemars(schema_with = "benchmark_attempt_v2_schema_version")]
    pub schema_version: String,
    pub identity: BenchmarkAttemptIdentity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes_attempt_id: Option<String>,
    pub transitions: Vec<BenchmarkAttemptTransition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCycleStatusV2 {
    #[schemars(schema_with = "benchmark_cycle_status_v2_schema_version")]
    pub schema_version: String,
    pub campaign_id: String,
    pub generated_at_utc: String,
    pub attempts: Vec<BenchmarkAttemptRecordV2>,
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

impl BenchmarkAttemptRecordV2 {
    #[must_use]
    pub fn planned(identity: BenchmarkAttemptIdentity, observed_at_utc: String) -> Self {
        Self {
            schema_version: BENCHMARK_ATTEMPT_SCHEMA_V2.to_string(),
            identity,
            supersedes_attempt_id: None,
            transitions: vec![BenchmarkAttemptTransition {
                sequence: 1,
                from_status: None,
                to_status: BenchmarkLifecycleStatus::Planned,
                observed_at_utc,
                reason: None,
                jobs: Vec::new(),
                evidence_paths: Vec::new(),
            }],
        }
    }

    #[must_use]
    pub fn current_status(&self) -> Option<BenchmarkLifecycleStatus> {
        self.transitions.last().map(|transition| transition.to_status)
    }
}

impl BenchmarkCycleStatusV2 {
    #[must_use]
    pub fn new(
        campaign_id: String,
        generated_at_utc: String,
        attempts: Vec<BenchmarkAttemptRecordV2>,
    ) -> Self {
        Self {
            schema_version: BENCHMARK_CYCLE_STATUS_SCHEMA_V2.to_string(),
            campaign_id,
            generated_at_utc,
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

fn benchmark_attempt_v2_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_ATTEMPT_SCHEMA_V2,
    })
}

fn benchmark_attempt_archive_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1,
    })
}

fn benchmark_attempt_migration_plan_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_ATTEMPT_MIGRATION_PLAN_SCHEMA_V1,
    })
}

fn benchmark_attempt_migration_ledger_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_ATTEMPT_MIGRATION_LEDGER_SCHEMA_V1,
    })
}

fn benchmark_cycle_status_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
    })
}

fn benchmark_cycle_status_v2_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_CYCLE_STATUS_SCHEMA_V2,
    })
}

fn sha256_identity_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^[0-9a-f]{64}$",
    })
}

fn study_uri_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^study://[^/].+$",
    })
}

fn archive_relative_path_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^(submission|result)/[^/].+$",
    })
}
