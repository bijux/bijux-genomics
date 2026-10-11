//! Owner: bijux-dna-bench-model
//! Typed catalogs for immutable data derived from governed scientific sources.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::CorpusSourcePolicy;
use crate::contract::GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDerivedDataSource {
    pub catalog_id: String,
    pub catalog_path: String,
    pub catalog_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDataDerivation {
    pub pipeline_id: String,
    pub pipeline_version: String,
    pub source_commit: String,
    pub parameter_sha256: String,
    pub reference_id: String,
    pub reference_catalog_sha256: String,
    pub contig_dictionary_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDerivedDataCatalogSample {
    pub sample_id: String,
    pub source_accessions: Vec<String>,
    pub library_context: String,
    pub assay_context: String,
    pub strata: BTreeMap<String, String>,
    pub control_roles: Vec<String>,
    pub truth_roles: Vec<String>,
    pub held_out_split: String,
    pub read_group_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDerivedDataCatalogFile {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub kind: String,
    pub sample_id: String,
    pub source_sha256s: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDerivedDataCatalog {
    #[schemars(schema_with = "derived_catalog_schema_version")]
    pub schema_version: String,
    pub id: String,
    pub purpose: String,
    pub source: String,
    #[serde(flatten)]
    pub policy: CorpusSourcePolicy,
    pub sources: Vec<GovernedDerivedDataSource>,
    pub admitted_at_utc: String,
    pub scientific_name: String,
    pub reference_assembly: String,
    pub allowed_uses: Vec<String>,
    pub derivation: GovernedDataDerivation,
    pub samples: Vec<GovernedDerivedDataCatalogSample>,
    pub files: Vec<GovernedDerivedDataCatalogFile>,
}

#[must_use]
pub fn governed_derived_data_catalog_schema() -> Schema {
    schemars::schema_for!(GovernedDerivedDataCatalog)
}

fn derived_catalog_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1})
}
