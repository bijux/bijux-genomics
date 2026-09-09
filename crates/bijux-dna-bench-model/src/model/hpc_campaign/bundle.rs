//! Persisted bundle provenance and declared transport integrity.
use schemars::{json_schema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};

pub const BUNDLE_SIDECAR_SCHEMA_VERSION: &str = "bijux.hpc.bundle.sidecar.v1";

#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BundleSidecar {
    #[serde(deserialize_with = "deserialize_version")]
    #[schemars(schema_with = "version_schema")]
    pub schema_version: String,
    pub bundle_kind: String,
    pub campaign_id: String,
    pub domain: String,
    pub stage: String,
    pub tool: String,
    pub sample: String,
    pub planned_job_id: String,
    pub scheduler_job_id: String,
    pub submitted_at: String,
    #[serde(deserialize_with = "deserialize_backend")]
    #[schemars(schema_with = "backend_schema")]
    pub backend: String,
    pub recipients: Vec<String>,
    pub recipient_fingerprints: Vec<String>,
    pub ciphertext_path: String,
    #[serde(deserialize_with = "deserialize_digest")]
    #[schemars(schema_with = "digest_schema")]
    pub plaintext_sha256: String,
    #[serde(deserialize_with = "deserialize_digest")]
    #[schemars(schema_with = "digest_schema")]
    pub ciphertext_sha256: String,
    pub plaintext_bytes: usize,
}

fn deserialize_version<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != BUNDLE_SIDECAR_SCHEMA_VERSION {
        return Err(serde::de::Error::custom("unsupported bundle sidecar schema"));
    }
    Ok(version)
}
fn deserialize_backend<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let backend = String::deserialize(deserializer)?;
    if !matches!(backend.as_str(), "mock-envelope-v1" | "age-cli") {
        return Err(serde::de::Error::custom("unsupported bundle encryption backend"));
    }
    Ok(backend)
}
fn deserialize_digest<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    let digest = String::deserialize(deserializer)?;
    if digest.len() != 64
        || !digest.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(serde::de::Error::custom(
            "bundle digest must be lowercase SHA-256 hexadecimal",
        ));
    }
    Ok(digest)
}
fn version_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const": BUNDLE_SIDECAR_SCHEMA_VERSION})
}
fn backend_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "enum": ["mock-envelope-v1", "age-cli"]})
}
fn digest_schema(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "pattern": "^[0-9a-f]{64}$"})
}
