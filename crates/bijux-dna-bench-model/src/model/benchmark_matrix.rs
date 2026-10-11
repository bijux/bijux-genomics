//! Owner: bijux-dna-bench-model
//! Persisted benchmark-matrix records consumed by campaign planning and appraisal.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

/// Stable schema identity retained for existing benchmark-matrix artifacts.
pub const BENCHMARK_MATRIX_REPORT_SCHEMA_V1: &str = "bijux.hpc.benchmark_matrix.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BenchmarkMatrixReport {
    #[serde(deserialize_with = "deserialize_benchmark_matrix_report_schema_version")]
    #[schemars(schema_with = "benchmark_matrix_report_schema_version")]
    pub schema_version: String,
    pub campaign_id: String,
    pub domain: String,
    pub domains: Vec<String>,
    pub generated_at: String,
    pub summary: BenchmarkMatrixSummary,
    pub rows: Vec<BenchmarkMatrixRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BenchmarkMatrixSummary {
    pub total_rows: usize,
    pub readiness_counts: BTreeMap<String, usize>,
    pub domain_counts: BTreeMap<String, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BenchmarkMatrixRow {
    pub row_id: String,
    pub matrix_domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_match: BenchmarkSurfaceMatch,
    pub database_match: BenchmarkSurfaceMatch,
    pub image_match: BenchmarkSurfaceMatch,
    pub readiness: BenchmarkReadiness,
    pub repetitions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BenchmarkSurfaceMatch {
    pub required_profile: String,
    pub matched_profile: String,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct BenchmarkReadiness {
    pub class: String,
    pub reasons: Vec<String>,
}

#[must_use]
pub fn benchmark_matrix_report_schema() -> Schema {
    schemars::schema_for!(BenchmarkMatrixReport)
}

fn benchmark_matrix_report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_MATRIX_REPORT_SCHEMA_V1,
    })
}

fn deserialize_benchmark_matrix_report_schema_version<'de, D>(
    deserializer: D,
) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == BENCHMARK_MATRIX_REPORT_SCHEMA_V1 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported benchmark matrix report schema: {value}")))
    }
}
