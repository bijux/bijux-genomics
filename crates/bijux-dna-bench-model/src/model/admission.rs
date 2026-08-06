//! Owner: bijux-dna-bench-model
//! Canonical persisted benchmark execution admission evidence.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use crate::contract::BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1;

use super::{
    BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceOperator, BenchmarkAttemptIdentity,
    BenchmarkEvidenceReference,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkOutputFileEvidence {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkMetricAssessment {
    pub id: String,
    pub path: String,
    pub json_pointer: String,
    pub operator: BenchmarkAcceptanceOperator,
    pub expected: BenchmarkAcceptanceExpectedValue,
    pub observed: BenchmarkAcceptanceExpectedValue,
    pub unit: String,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionAdmission {
    #[serde(deserialize_with = "deserialize_admission_schema_version")]
    #[schemars(schema_with = "admission_schema_version")]
    pub schema_version: String,
    pub identity: BenchmarkAttemptIdentity,
    pub admitted_at_utc: String,
    pub execution_spec_sha256: String,
    pub completion_sha256: String,
    pub output_manifest_sha256: String,
    pub downstream_probe: BenchmarkOutputFileEvidence,
    pub validation_files: Vec<BenchmarkOutputFileEvidence>,
    pub outputs: Vec<BenchmarkOutputFileEvidence>,
    pub metrics: Vec<BenchmarkMetricAssessment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BenchmarkExecutionAdmissionPublication {
    pub report: BenchmarkExecutionAdmission,
    pub evidence: BenchmarkEvidenceReference,
}

#[must_use]
pub fn benchmark_execution_admission_schema() -> Schema {
    schemars::schema_for!(BenchmarkExecutionAdmission)
}

fn admission_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1,
    })
}

fn deserialize_admission_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported benchmark execution admission: {value}")))
    }
}
