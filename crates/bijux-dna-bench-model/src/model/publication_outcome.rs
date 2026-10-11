//! Owner: bijux-dna-bench-model
//! Persisted outcomes emitted by governed benchmark publication workflows.

use std::path::PathBuf;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::contract::{
    BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_REGISTRY_PUBLICATION_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1,
};

use super::GovernedDataCatalogRegistry;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDataCatalogSnapshotPublication {
    #[serde(deserialize_with = "deserialize_catalog_snapshot_publication_schema_version")]
    #[schemars(schema_with = "catalog_snapshot_publication_schema_version")]
    pub schema_version: String,
    pub candidate_registry_path: PathBuf,
    pub preserved_snapshot_count: u64,
    pub retained_snapshot_count: u64,
    pub registry: GovernedDataCatalogRegistry,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GovernedDataCatalogRegistryPublication {
    #[serde(deserialize_with = "deserialize_catalog_registry_publication_schema_version")]
    #[schemars(schema_with = "catalog_registry_publication_schema_version")]
    pub schema_version: String,
    pub registry_path: PathBuf,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub registry_sha256: String,
    pub registry: GovernedDataCatalogRegistry,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCampaignReportPublication {
    #[serde(deserialize_with = "deserialize_campaign_report_publication_schema_version")]
    #[schemars(schema_with = "campaign_report_publication_schema_version")]
    pub schema_version: String,
    pub run_id: String,
    pub stage_id: String,
    pub report_path: PathBuf,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub report_sha256: String,
    pub markdown_path: PathBuf,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub markdown_sha256: String,
    #[schemars(schema_with = "repository_commit_schema")]
    pub presentation_repository_commit: String,
    #[schemars(schema_with = "sha256_identity_schema")]
    pub presentation_catalog_sha256: String,
    pub presentation_overlaid_stage_ids: Vec<String>,
}

#[must_use]
pub fn governed_data_catalog_snapshot_publication_schema() -> Schema {
    schemars::schema_for!(GovernedDataCatalogSnapshotPublication)
}

#[must_use]
pub fn governed_data_catalog_registry_publication_schema() -> Schema {
    schemars::schema_for!(GovernedDataCatalogRegistryPublication)
}

#[must_use]
pub fn benchmark_campaign_report_publication_schema() -> Schema {
    schemars::schema_for!(BenchmarkCampaignReportPublication)
}

fn catalog_snapshot_publication_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1,
    })
}

fn catalog_registry_publication_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": GOVERNED_DATA_CATALOG_REGISTRY_PUBLICATION_SCHEMA_V1,
    })
}

fn campaign_report_publication_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1,
    })
}

fn sha256_identity_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "pattern": "^[0-9a-f]{64}$"})
}

fn repository_commit_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "pattern": "^[0-9a-f]{40}$"})
}

fn deserialize_catalog_snapshot_publication_schema_version<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_schema_version(
        deserializer,
        GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1,
        "governed data catalog snapshot publication",
    )
}

fn deserialize_catalog_registry_publication_schema_version<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_schema_version(
        deserializer,
        GOVERNED_DATA_CATALOG_REGISTRY_PUBLICATION_SCHEMA_V1,
        "governed data catalog registry publication",
    )
}

fn deserialize_campaign_report_publication_schema_version<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_schema_version(
        deserializer,
        BENCHMARK_CAMPAIGN_REPORT_PUBLICATION_SCHEMA_V1,
        "benchmark campaign report publication",
    )
}

fn deserialize_schema_version<'de, D>(
    deserializer: D,
    expected: &str,
    label: &str,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == expected {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported {label} schema: {value}")))
    }
}
