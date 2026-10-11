//! Owner: bijux-dna-bench
//! Typed benchmark execution admission evidence.

use bijux_dna_bench_model::{
    BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceOperator, BenchmarkAttemptIdentity,
    BenchmarkEvidenceReference,
};
use serde::{Deserialize, Serialize};

pub const BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1: &str = "bijux.benchmark.execution_admission.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkOutputFileEvidence {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkExecutionAdmission {
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
