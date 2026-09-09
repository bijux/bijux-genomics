//! Persisted campaign copy-back instructions; these do not assert that files were transferred.
use schemars::{json_schema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
pub const COPY_BACK_MANIFEST_SCHEMA_VERSION: &str = "bijux.hpc.copy_back_manifest.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CopyBackManifestReport {
    #[serde(deserialize_with = "deserialize_version")]
    #[schemars(schema_with = "version_schema")]
    pub schema_version: String,
    pub manifest_path: String,
    pub campaign_id: String,
    pub domain: String,
    pub suggested_copy_command: String,
    pub entries: Vec<CopyBackEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CopyBackEntry {
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub scratch_dir: String,
    pub log_path: String,
    pub out_path: String,
    pub err_path: String,
    pub results_path: String,
    pub results_sidecar_path: String,
    pub code_path: String,
    pub code_sidecar_path: String,
    pub script_path: String,
}

fn deserialize_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != COPY_BACK_MANIFEST_SCHEMA_VERSION {
        return Err(serde::de::Error::custom("unsupported copy-back manifest schema"));
    }
    Ok(version)
}
fn version_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const": COPY_BACK_MANIFEST_SCHEMA_VERSION})
}
