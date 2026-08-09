//! Paper-facing benchmark run index and selected-publication references.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_RUN_INDEX_SCHEMA_V1;

use super::{BenchmarkAttemptIdentity, BenchmarkRunRegistryState};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRunIndexFileReference {
    #[schemars(schema_with = "evidence_uri_schema")]
    pub uri: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRunIndexPublication {
    pub publication_id: String,
    pub selected_attempts: Vec<BenchmarkAttemptIdentity>,
    #[schemars(schema_with = "repository_commit_schema")]
    pub repository_commit: String,
    pub report: BenchmarkRunIndexFileReference,
    pub evidence_book: BenchmarkRunIndexFileReference,
    pub publication_receipt: BenchmarkRunIndexFileReference,
    pub citation_record: BenchmarkRunIndexFileReference,
    pub replay_receipt: BenchmarkRunIndexFileReference,
    pub limitations: Vec<String>,
    pub access_statement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRunIndexEntry {
    pub run_id: String,
    pub citation_key: String,
    pub protocol_title: String,
    pub slug: String,
    pub stage_id: String,
    pub owner: String,
    pub protocol_state: BenchmarkRunRegistryState,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub protocol_sha256: String,
    pub registry_uri: String,
    pub tool_ids: Vec<String>,
    pub corpus_roles: Vec<String>,
    pub acceptance_policy_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_publication: Option<BenchmarkRunIndexPublication>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRunIndexSummary {
    pub registered_run_count: usize,
    pub active_protocol_count: usize,
    pub reserved_protocol_count: usize,
    pub retired_protocol_count: usize,
    pub citable_publication_count: usize,
    pub unpublished_run_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkRunIndex {
    #[schemars(schema_with = "run_index_schema_version")]
    pub schema_version: String,
    pub source_registry: BenchmarkRunIndexFileReference,
    pub summary: BenchmarkRunIndexSummary,
    pub runs: Vec<BenchmarkRunIndexEntry>,
}

impl BenchmarkRunIndex {
    #[must_use]
    pub fn v1(
        source_registry: BenchmarkRunIndexFileReference,
        summary: BenchmarkRunIndexSummary,
        runs: Vec<BenchmarkRunIndexEntry>,
    ) -> Self {
        Self {
            schema_version: BENCHMARK_RUN_INDEX_SCHEMA_V1.to_string(),
            source_registry,
            summary,
            runs,
        }
    }
}

#[must_use]
pub fn benchmark_run_index_schema() -> Schema {
    schemars::schema_for!(BenchmarkRunIndex)
}

fn run_index_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_RUN_INDEX_SCHEMA_V1,
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

fn evidence_uri_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^(repo|study)://[^/].+$",
    })
}
