//! Closed wire contracts for illustrative all-domain failure records and their manifest.
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

pub const DEFAULT_ALL_DOMAIN_FAKE_FAILURE_ROOT: &str =
    "artifacts/benchmark-smoke/simulated-runs/all-domains-failures";
pub const ALL_DOMAIN_FAKE_FAILURE_MANIFEST_SCHEMA_VERSION: &str =
    "bijux.bench.local_all_domain_fake_failures.v1";
pub const ALL_DOMAIN_FAKE_FAILURE_RECORD_SCHEMA_VERSION: &str =
    "bijux.bench.local_all_domain_fake_failure_record.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainFakeFailureOutputEntry {
    pub artifact_id: String,
    pub declared_output: String,
    pub expected_fake_run_path: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainFakeFailureRecord {
    #[serde(deserialize_with = "deserialize_record_version")]
    #[schemars(schema_with = "record_version")]
    pub schema_version: String,
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub command_source: String,
    pub command: String,
    pub exit_code: i32,
    pub command_script_path: String,
    pub stderr_path: String,
    pub failure_record_path: String,
    pub failed_output_count: usize,
    pub failed_outputs: Vec<AllDomainFakeFailureOutputEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AllDomainFakeFailuresManifest {
    #[serde(deserialize_with = "deserialize_manifest_version")]
    #[schemars(schema_with = "manifest_version")]
    pub schema_version: String,
    pub failure_root: String,
    pub root_manifest_path: String,
    pub result_count: usize,
    pub failed_output_count: usize,
    pub exit_code: i32,
    pub domain_counts: BTreeMap<String, usize>,
    pub failures: Vec<AllDomainFakeFailureRecord>,
}

#[must_use]
pub fn all_domain_fake_failure_manifest_schema() -> Schema {
    schemars::schema_for!(AllDomainFakeFailuresManifest)
}
#[must_use]
pub fn all_domain_fake_failure_record_schema() -> Schema {
    schemars::schema_for!(AllDomainFakeFailureRecord)
}
fn manifest_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":ALL_DOMAIN_FAKE_FAILURE_MANIFEST_SCHEMA_VERSION})
}
fn record_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":ALL_DOMAIN_FAKE_FAILURE_RECORD_SCHEMA_VERSION})
}
fn deserialize_manifest_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    deserialize_version(deserializer, ALL_DOMAIN_FAKE_FAILURE_MANIFEST_SCHEMA_VERSION)
}
fn deserialize_record_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    deserialize_version(deserializer, ALL_DOMAIN_FAKE_FAILURE_RECORD_SCHEMA_VERSION)
}
fn deserialize_version<'de, D: Deserializer<'de>>(
    deserializer: D,
    expected: &str,
) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != expected {
        return Err(serde::de::Error::custom(format!(
            "unsupported failure simulation schema {version}"
        )));
    }
    Ok(version)
}
