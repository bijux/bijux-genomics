//! Semantic replay receipts for immutable benchmark publications.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1;

use super::{BenchmarkAttemptIdentity, BenchmarkEvidenceReference, BenchmarkRunIdentity};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkReplayOutcome {
    ExactMatch,
    Mismatch,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkReplaySemanticIdentity {
    #[schemars(schema_with = "sha256_identity_schema")]
    pub decision_sha256: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub report_sha256: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub evidence_book_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkReplayReceipt {
    #[schemars(schema_with = "benchmark_replay_receipt_schema_version")]
    pub schema_version: String,
    pub replay_id: String,
    pub replayed_at_utc: String,
    pub run: BenchmarkRunIdentity,
    pub publication_id: String,
    pub selected_attempts: Vec<BenchmarkAttemptIdentity>,
    #[schemars(schema_with = "repository_commit_schema")]
    pub source_repository_commit: String,
    #[schemars(schema_with = "repository_commit_schema")]
    pub replay_repository_commit: String,
    pub source_publication: BenchmarkEvidenceReference,
    pub run_definition: BenchmarkEvidenceReference,
    pub case_definitions: Vec<BenchmarkEvidenceReference>,
    pub immutable_inputs: Vec<BenchmarkEvidenceReference>,
    pub expected: BenchmarkReplaySemanticIdentity,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed: Option<BenchmarkReplaySemanticIdentity>,
    pub outcome: BenchmarkReplayOutcome,
    pub differences: Vec<String>,
    pub limitations: Vec<String>,
}

#[must_use]
pub fn benchmark_replay_receipt_schema() -> Schema {
    schemars::schema_for!(BenchmarkReplayReceipt)
}

fn benchmark_replay_receipt_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_REPLAY_RECEIPT_SCHEMA_V1,
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
