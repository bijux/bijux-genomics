//! Permanent benchmark run protocols and source-locked executable cases.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{BENCHMARK_CASE_DEFINITION_SCHEMA_V1, BENCHMARK_RUN_DEFINITION_SCHEMA_V1};

use super::BenchmarkRunIdentity;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkRunState {
    Active,
    Reserved,
    Retired,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkContractLock {
    pub contract_id: String,
    pub repository_path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusRole {
    pub role: String,
    pub corpus_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRunDefinition {
    #[schemars(schema_with = "benchmark_run_definition_schema_version")]
    pub schema_version: String,
    pub identity: BenchmarkRunIdentity,
    #[schemars(schema_with = "protocol_version_schema")]
    pub protocol_version: String,
    pub title: String,
    pub purpose: String,
    pub owner: String,
    pub state: BenchmarkRunState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state_reason: Option<String>,
    pub tool_ids: Vec<String>,
    pub contracts: Vec<BenchmarkContractLock>,
    pub corpus_roles: Vec<BenchmarkCorpusRole>,
    pub acceptance_policy_id: String,
    pub citation_key: String,
    pub supersedes_run_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseInputLock {
    pub role: String,
    pub asset_id: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseQualificationReceipt {
    pub tool_id: String,
    #[schemars(schema_with = "study_uri_schema")]
    pub uri: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub receipt_sha256: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub image_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseDefinition {
    #[schemars(schema_with = "benchmark_case_definition_schema_version")]
    pub schema_version: String,
    pub case_id: String,
    pub campaign_id: String,
    pub run: BenchmarkRunIdentity,
    pub tool_id: String,
    #[schemars(schema_with = "repository_commit_schema")]
    pub repository_commit: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub corpus_snapshot_sha256: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub parameter_lock_sha256: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub execution_spec_sha256: String,
    pub inputs: Vec<BenchmarkCaseInputLock>,
    pub qualification: BenchmarkCaseQualificationReceipt,
    pub warmup_repetitions: u32,
    pub measured_repetitions: u32,
}

#[must_use]
pub fn benchmark_run_definition_schema() -> Schema {
    schemars::schema_for!(BenchmarkRunDefinition)
}

#[must_use]
pub fn benchmark_case_definition_schema() -> Schema {
    schemars::schema_for!(BenchmarkCaseDefinition)
}

fn benchmark_run_definition_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_RUN_DEFINITION_SCHEMA_V1,
    })
}

fn benchmark_case_definition_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_CASE_DEFINITION_SCHEMA_V1,
    })
}

fn protocol_version_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^(0|[1-9][0-9]*)[.](0|[1-9][0-9]*)[.](0|[1-9][0-9]*)$",
    })
}

fn sha256_identity_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^[0-9a-f]{64}$",
    })
}

fn repository_commit_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^[0-9a-f]{40}$",
    })
}

fn study_uri_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^study://[^/].+$",
    })
}
