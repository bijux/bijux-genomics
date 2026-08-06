//! Immutable benchmark publication selection and evidence-book records.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1, BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1};

use super::{BenchmarkAttemptIdentity, BenchmarkEvidenceReference, BenchmarkRunIdentity};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPublicationReceipt {
    #[schemars(schema_with = "benchmark_publication_receipt_schema_version")]
    pub schema_version: String,
    pub publication_id: String,
    pub published_at_utc: String,
    pub run: BenchmarkRunIdentity,
    pub case_ids: Vec<String>,
    pub selected_attempts: Vec<BenchmarkAttemptIdentity>,
    #[schemars(schema_with = "repository_commit_schema")]
    pub repository_commit: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub catalog_sha256: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub run_definition_sha256: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub campaign_registry_sha256: String,
    pub report: BenchmarkEvidenceReference,
    pub decision: BenchmarkEvidenceReference,
    pub evidence: Vec<BenchmarkEvidenceReference>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkEvidenceBookStatus {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkEvidenceBookEntry {
    pub publication_id: String,
    pub run: BenchmarkRunIdentity,
    pub citation_key: String,
    pub publication_receipt: BenchmarkEvidenceReference,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkEvidenceBook {
    #[schemars(schema_with = "benchmark_evidence_book_schema_version")]
    pub schema_version: String,
    pub book_id: String,
    pub generated_at_utc: String,
    #[schemars(schema_with = "repository_commit_schema")]
    pub repository_commit: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub catalog_sha256: String,
    pub status: BenchmarkEvidenceBookStatus,
    pub entries: Vec<BenchmarkEvidenceBookEntry>,
    pub missing_run_ids: Vec<String>,
    pub limitations: Vec<String>,
}

#[must_use]
pub fn benchmark_publication_receipt_schema() -> Schema {
    schemars::schema_for!(BenchmarkPublicationReceipt)
}

#[must_use]
pub fn benchmark_evidence_book_schema() -> Schema {
    schemars::schema_for!(BenchmarkEvidenceBook)
}

fn benchmark_publication_receipt_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_PUBLICATION_RECEIPT_SCHEMA_V1,
    })
}

fn benchmark_evidence_book_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_EVIDENCE_BOOK_SCHEMA_V1,
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
