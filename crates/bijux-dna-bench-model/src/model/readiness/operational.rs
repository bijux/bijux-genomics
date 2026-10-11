//! Persisted operational readiness observations and decisions.

use schemars::{json_schema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize};

pub const OPERATIONAL_BENCHMARK_READY_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.operational_benchmark_ready.v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationalBenchmarkReadyCheck {
    pub surface_id: String,
    pub output_path: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Serialize,
    serde::Deserialize,
    schemars::JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct OperationalBenchmarkReadyBlocker {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub asset_profile_id: String,
    pub blocker_type: String,
    pub blocker_path: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OperationalBenchmarkReadyReport {
    #[serde(deserialize_with = "deserialize_schema_version")]
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub checked_surface_count: usize,
    pub passed_surface_count: usize,
    pub failed_surface_count: usize,
    pub benchmark_ready_row_count: usize,
    pub blocker_count: usize,
    pub missing_result_row_count: usize,
    pub insufficient_data_row_count: usize,
    pub unsupported_pair_row_count: usize,
    pub ok: bool,
    pub checks: Vec<OperationalBenchmarkReadyCheck>,
    pub blockers: Vec<OperationalBenchmarkReadyBlocker>,
}

/// Counts observed from expected results and the admitted result collector.
#[derive(Debug, Clone, Copy, Default)]
pub struct OperationalBenchmarkResultCounts {
    pub benchmark_ready: usize,
    pub missing: usize,
    pub insufficient: usize,
    pub unsupported: usize,
}

fn deserialize_schema_version<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<String, D::Error> {
    let version = String::deserialize(deserializer)?;
    if version != OPERATIONAL_BENCHMARK_READY_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(
            "unsupported operational benchmark readiness schema version",
        ));
    }
    Ok(version)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type":"string", "const":OPERATIONAL_BENCHMARK_READY_SCHEMA_VERSION})
}
