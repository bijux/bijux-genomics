//! Owner: bijux-dna-bench-model
//! Persisted active-scope validation records.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

pub const ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1: &str =
    "bijux.bench.active_scope_validate_fast.v1";
pub(crate) const ACTIVE_SCOPE_FAST_VALIDATION_MODE: &str = "fast";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ActiveScopeFastValidationCheck {
    pub category: String,
    pub surface_id: String,
    pub output_path: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ActiveScopeFastValidationReport {
    #[serde(deserialize_with = "deserialize_schema_version")]
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    #[serde(deserialize_with = "deserialize_mode")]
    #[schemars(schema_with = "mode")]
    pub mode: String,
    pub benchmark_root: String,
    pub schema_root: String,
    pub fixture_root: String,
    pub checked_surface_count: usize,
    pub passed_surface_count: usize,
    pub failed_surface_count: usize,
    pub category_counts: BTreeMap<String, usize>,
    pub failed_category_counts: BTreeMap<String, usize>,
    pub ok: bool,
    pub checks: Vec<ActiveScopeFastValidationCheck>,
}

#[must_use]
pub fn active_scope_fast_validation_report_schema() -> Schema {
    schemars::schema_for!(ActiveScopeFastValidationReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1,
    })
}

fn mode(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": ACTIVE_SCOPE_FAST_VALIDATION_MODE,
    })
}

fn deserialize_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported active-scope fast validation schema: {value}")))
    }
}

fn deserialize_mode<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == ACTIVE_SCOPE_FAST_VALIDATION_MODE {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported active-scope validation mode: {value}")))
    }
}
