//! Owner: bijux-dna-bench-model
//! Persisted audit of benchmark portfolio bindings to governed scientific data.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2;
use crate::model::{BenchmarkCorpusClass, BenchmarkCorpusStatus, BenchmarkEvidenceScope};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataSourceSpecIdentity {
    pub path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub sha256: String,
    pub projected_storage_bytes: Option<u64>,
    pub projected_cpu_hours: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkGovernedDataBinding {
    pub catalog_id: String,
    pub present: bool,
    pub catalog_path: Option<String>,
    #[schemars(schema_with = "optional_sha256_identity_schema")]
    pub catalog_sha256: Option<String>,
    pub access_classification: Option<String>,
    pub redistribution_classification: Option<String>,
    pub access_terms: Option<String>,
    pub integrity_verified: bool,
    pub benchmark_eligible: bool,
    pub verified_file_count: u64,
    pub verified_size_bytes: u64,
    pub observed_sample_count: Option<u64>,
    pub sample_count_authority: Option<String>,
    pub file_roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusDataAudit {
    pub corpus_id: String,
    pub status: BenchmarkCorpusStatus,
    pub evidence_scope: BenchmarkEvidenceScope,
    pub class: BenchmarkCorpusClass,
    pub declared_sample_count: u32,
    pub minimum_sample_count: u32,
    pub data_types: Vec<String>,
    pub truth_roles: Vec<String>,
    pub truth_strategy: String,
    pub source_specs: Vec<BenchmarkDataSourceSpecIdentity>,
    pub catalog_bindings: Vec<BenchmarkGovernedDataBinding>,
    pub verified_storage_bytes: u64,
    pub projected_storage_bytes: Option<u64>,
    pub projected_cpu_hours: Option<u64>,
    pub publishable_data_ready: bool,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkUnboundDataAsset {
    pub catalog_id: String,
    pub purpose: String,
    pub access_classification: String,
    pub redistribution_classification: String,
    pub verified_file_count: u64,
    pub verified_size_bytes: u64,
    pub suspected_nonproduction: bool,
    pub classification_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataPortfolioAuditSummary {
    pub corpus_count: u64,
    pub admitted_corpus_count: u64,
    pub source_defined_corpus_count: u64,
    pub missing_corpus_count: u64,
    pub publishable_data_ready_count: u64,
    pub bound_catalog_count: u64,
    pub missing_catalog_binding_count: u64,
    pub unbound_catalog_count: u64,
    pub verified_bound_file_count: u64,
    pub verified_bound_size_bytes: u64,
    pub unknown_storage_projection_count: u64,
    pub unknown_compute_projection_count: u64,
    pub suspected_nonproduction_asset_count: u64,
    pub publishable_contamination_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataPortfolioAudit {
    #[schemars(schema_with = "data_portfolio_audit_schema_version")]
    pub schema_version: String,
    #[schemars(schema_with = "git_commit_identity_schema")]
    pub source_commit: String,
    pub portfolio_path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub portfolio_sha256: String,
    pub data_index_path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub data_index_sha256: String,
    pub summary: BenchmarkDataPortfolioAuditSummary,
    pub corpora: Vec<BenchmarkCorpusDataAudit>,
    pub unbound_assets: Vec<BenchmarkUnboundDataAsset>,
    pub publishable_contamination: Vec<String>,
}

#[must_use]
pub fn benchmark_data_portfolio_audit_schema() -> Schema {
    schemars::schema_for!(BenchmarkDataPortfolioAudit)
}

fn data_portfolio_audit_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2,
    })
}

fn sha256_identity_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "pattern": "^[0-9a-f]{64}$"})
}

fn optional_sha256_identity_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "oneOf": [
            {"type": "string", "pattern": "^[0-9a-f]{64}$"},
            {"type": "null"}
        ]
    })
}

fn git_commit_identity_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "pattern": "^[0-9a-f]{40}$"})
}
