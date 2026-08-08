//! Owner: bijux-dna-bench-model
//! Persisted benchmark publication authority and lifecycle classification.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_PUBLICATION_REGISTRY_SCHEMA_V1;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStagePublicationDisposition {
    Active,
    Historical,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCurrentPublication {
    pub product_id: String,
    pub json_path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub json_sha256: String,
    pub markdown_path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub markdown_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStagePublication {
    pub run_id: String,
    pub stage_id: String,
    #[schemars(schema_with = "git_commit_identity_schema")]
    pub source_commit: String,
    pub disposition: BenchmarkStagePublicationDisposition,
    pub json_path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub json_sha256: String,
    pub markdown_path: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub markdown_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkQuarantinedPublicationSurface {
    pub path_prefix: String,
    pub disposition: String,
    pub tracked_file_count: usize,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub tree_sha256: String,
    pub authority_policy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPublicationRegistrySummary {
    pub current_product_count: usize,
    pub active_stage_report_count: usize,
    pub historical_stage_report_count: usize,
    pub quarantined_surface_count: usize,
    pub quarantined_file_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPublicationRegistry {
    #[schemars(schema_with = "publication_registry_schema_version")]
    pub schema_version: String,
    pub summary: BenchmarkPublicationRegistrySummary,
    pub current_products: Vec<BenchmarkCurrentPublication>,
    pub stage_reports: Vec<BenchmarkStagePublication>,
    pub quarantined_surfaces: Vec<BenchmarkQuarantinedPublicationSurface>,
}

impl BenchmarkPublicationRegistry {
    #[must_use]
    pub fn v1(
        summary: BenchmarkPublicationRegistrySummary,
        current_products: Vec<BenchmarkCurrentPublication>,
        stage_reports: Vec<BenchmarkStagePublication>,
        quarantined_surfaces: Vec<BenchmarkQuarantinedPublicationSurface>,
    ) -> Self {
        Self {
            schema_version: BENCHMARK_PUBLICATION_REGISTRY_SCHEMA_V1.to_string(),
            summary,
            current_products,
            stage_reports,
            quarantined_surfaces,
        }
    }
}

#[must_use]
pub fn benchmark_publication_registry_schema() -> Schema {
    schemars::schema_for!(BenchmarkPublicationRegistry)
}

fn publication_registry_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_PUBLICATION_REGISTRY_SCHEMA_V1,
    })
}

fn sha256_identity_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^[0-9a-f]{64}$",
    })
}

fn git_commit_identity_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": "^[0-9a-f]{40}$",
    })
}
