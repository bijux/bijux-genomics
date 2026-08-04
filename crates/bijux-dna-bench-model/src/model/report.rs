//! Owner: bijux-dna-bench-model
//! Canonical benchmark stage-report contracts and historical import fields.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_STAGE_REPORT_SCHEMA_V1;

use super::planning::{
    BenchmarkCachePolicy, BenchmarkEvidenceTier, BenchmarkExecutionRuntime,
    BenchmarkExecutionScheduler,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkReportVerdict {
    Passed,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkComparisonStatus {
    NotApplicable,
    WithinTolerance,
    OutsideTolerance,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(untagged)]
pub enum BenchmarkScientificValue {
    Number(f64),
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkMetricDistribution {
    pub minimum: f64,
    pub maximum: f64,
    pub mean: f64,
    pub sample_standard_deviation: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkReportImage {
    pub name: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkReportSlurmJob {
    pub job_id: String,
    pub host: String,
    pub elapsed: String,
    pub allocated_cpus: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkReportDeterminism {
    pub artifact: String,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub digest_basis: Option<String>,
    pub identical_across_repetitions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkReportInputLock {
    pub id: String,
    pub kind: String,
    pub mount_path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkToolReport {
    pub tool: String,
    pub verdict: BenchmarkReportVerdict,
    pub repository_commit: String,
    pub image: BenchmarkReportImage,
    pub slurm: BenchmarkReportSlurmJob,
    pub source_evidence_sha256: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_locks: Vec<BenchmarkReportInputLock>,
    pub input_bytes: BTreeMap<String, u64>,
    pub performance: BTreeMap<String, BenchmarkMetricDistribution>,
    pub scientific_observations: BTreeMap<String, Vec<BenchmarkScientificValue>>,
    pub determinism: BenchmarkReportDeterminism,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageReportDesign {
    pub runtime: BenchmarkExecutionRuntime,
    pub scheduler: BenchmarkExecutionScheduler,
    pub container_only: bool,
    pub threads_by_tool: BTreeMap<String, u32>,
    pub cache_policy: BenchmarkCachePolicy,
    pub warmup_repetitions: u32,
    pub measured_repetitions: u32,
    pub tool_closure: Vec<String>,
    pub corpus_ids: Vec<String>,
    pub corpus_catalog_ids: Vec<String>,
    pub reference_catalog_ids: Vec<String>,
    pub database_catalog_ids: Vec<String>,
    pub input_size_keys: Vec<String>,
    pub result_artifact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkReportDownstreamProbe {
    pub stage_id: String,
    pub input_contract: String,
    pub source: String,
    pub artifact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkReportStageInterface {
    pub accepted_input_contracts: Vec<String>,
    pub output_contract: String,
    pub normalized_result_contract: String,
    pub downstream_probe: BenchmarkReportDownstreamProbe,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkReportComparability {
    pub repository_commit: String,
    pub input_lock_sha256: String,
    pub parameter_lock_sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage_interface: Option<BenchmarkReportStageInterface>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkComparisonMetric {
    pub metric: String,
    pub observations: BTreeMap<String, f64>,
    pub minimum: f64,
    pub maximum: f64,
    pub absolute_range: f64,
    pub allowed_range: f64,
    pub within_tolerance: bool,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCrossToolComparison {
    pub status: BenchmarkComparisonStatus,
    pub metrics: Vec<BenchmarkComparisonMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageReport {
    #[schemars(schema_with = "benchmark_stage_report_schema_version")]
    pub schema_version: String,
    pub verdict: BenchmarkReportVerdict,
    pub run_id: String,
    pub stage_id: String,
    pub evidence_tier: BenchmarkEvidenceTier,
    pub benchmark_profile_id: String,
    pub scientific_question: String,
    pub comparison_scope: String,
    pub accuracy_claim: String,
    pub design: BenchmarkStageReportDesign,
    pub tools: Vec<BenchmarkToolReport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub comparability: Option<BenchmarkReportComparability>,
    pub cross_tool_comparison: BenchmarkCrossToolComparison,
    pub limitations: Vec<String>,
    pub interpretation: String,
}

fn benchmark_stage_report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_STAGE_REPORT_SCHEMA_V1,
    })
}
