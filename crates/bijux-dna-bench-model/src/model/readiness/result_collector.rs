//! Persisted rows and reports for the full benchmark result collector.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FullBenchmarkResultSurfaceKind {
    BenchmarkExpected,
    PipelineFakeRun,
    FakeRun,
    FakeFailure,
    MissingResultAudit,
    RealSmoke,
    FailureClassification,
    UnsupportedPair,
}

impl FullBenchmarkResultSurfaceKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BenchmarkExpected => "benchmark_expected",
            Self::PipelineFakeRun => "pipeline_fake_run",
            Self::FakeRun => "fake_run",
            Self::FakeFailure => "fake_failure",
            Self::MissingResultAudit => "missing_result_audit",
            Self::RealSmoke => "real_smoke",
            Self::FailureClassification => "failure_classification",
            Self::UnsupportedPair => "unsupported_pair",
        }
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum FullBenchmarkResultStatus {
    Expected,
    Succeeded,
    Failed,
    Present,
    MissingResult,
    InsufficientData,
    UnsupportedPair,
}

impl FullBenchmarkResultStatus {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expected => "expected",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Present => "present",
            Self::MissingResult => "missing_result",
            Self::InsufficientData => "insufficient_data",
            Self::UnsupportedPair => "unsupported_pair",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkResultCollectorRow {
    pub record_id: String,
    pub source_surface: String,
    pub surface_kind: FullBenchmarkResultSurfaceKind,
    pub result_status: FullBenchmarkResultStatus,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub pipeline_id: Option<String>,
    pub node_id: Option<String>,
    pub result_id: Option<String>,
    pub execution_id: Option<String>,
    pub corpus_id: Option<String>,
    pub asset_profile_id: Option<String>,
    pub report_section: Option<String>,
    pub evidence_path: String,
    pub manifest_path: Option<String>,
    pub declared_output_count: usize,
    pub normalized_metric_count: usize,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FullBenchmarkResultCollectorReport {
    #[schemars(schema_with = "schema_version")]
    pub schema_version: String,
    pub output_path: String,
    pub row_count: usize,
    pub benchmark_expected_row_count: usize,
    pub pipeline_fake_run_row_count: usize,
    pub fake_run_row_count: usize,
    pub fake_failure_row_count: usize,
    pub missing_result_audit_row_count: usize,
    pub real_smoke_row_count: usize,
    pub insufficient_data_row_count: usize,
    pub unsupported_pair_row_count: usize,
    pub missing_result_status_count: usize,
    pub insufficient_data_status_count: usize,
    pub unsupported_pair_status_count: usize,
    pub passes_behavior_test: bool,
    pub surface_kind_counts: BTreeMap<String, usize>,
    pub result_status_counts: BTreeMap<String, usize>,
    pub domain_counts: BTreeMap<String, usize>,
    pub rows: Vec<FullBenchmarkResultCollectorRow>,
}

#[must_use]
pub fn full_benchmark_result_collector_schema() -> Schema {
    schemars::schema_for!(FullBenchmarkResultCollectorReport)
}

fn schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": FULL_BENCHMARK_RESULT_COLLECTOR_SCHEMA_V1})
}
