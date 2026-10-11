//! Owner: bijux-dna-bench-model
//! Immutable catalog snapshot selection for governed benchmark data.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDataCatalogSnapshot {
    pub catalog_path: String,
    pub catalog_sha256: String,
    pub catalog_schema_version: String,
    pub source_commit: String,
    pub created_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDataCatalogRegistryRecord {
    pub id: String,
    pub current_catalog_sha256: String,
    pub snapshots: Vec<GovernedDataCatalogSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDataCatalogRegistry {
    #[schemars(schema_with = "catalog_registry_schema_version")]
    pub schema_version: String,
    pub created_at_utc: String,
    pub data_root: String,
    pub records: Vec<GovernedDataCatalogRegistryRecord>,
}

#[must_use]
pub fn governed_data_catalog_registry_schema() -> Schema {
    schemars::schema_for!(GovernedDataCatalogRegistry)
}

fn catalog_registry_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1})
}
