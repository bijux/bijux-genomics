//! Owner: bijux-dna-bench-model
//! Domain-neutral source authorities for production benchmark corpora.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_DATA_SOURCE_SCHEMA_V1;
use crate::model::{CorpusEvaluationDesign, CorpusSourcePolicy};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataSourceCollection {
    pub id: String,
    pub authority: String,
    pub accession: String,
    pub source_url: String,
    pub access_terms_url: String,
    pub data_types: Vec<String>,
    pub planned_sample_count: Option<u64>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_accessions: Vec<String>,
    pub selection_scope: String,
    pub truth_basis: String,
    pub known_limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataSourceSpec {
    #[schemars(schema_with = "benchmark_data_source_schema_version")]
    pub schema_version: String,
    pub portfolio_id: String,
    #[serde(flatten)]
    pub policy: CorpusSourcePolicy,
    #[serde(flatten)]
    pub evaluation: CorpusEvaluationDesign,
    pub description: String,
    pub data_types: Vec<String>,
    pub target_sample_count: u64,
    pub collections: Vec<BenchmarkDataSourceCollection>,
}

#[must_use]
pub fn benchmark_data_source_spec_schema() -> Schema {
    schemars::schema_for!(BenchmarkDataSourceSpec)
}

fn benchmark_data_source_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BENCHMARK_DATA_SOURCE_SCHEMA_V1})
}
