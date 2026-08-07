//! Owner: bijux-dna-bench
//! Persistable case-matrix contracts for registry-driven benchmark execution.

use bijux_dna_bench_model::{
    BenchmarkCachePolicy, BenchmarkComparisonMetricTolerance, BenchmarkEvidenceTier,
    BenchmarkStageInterfaceSpec,
};
use bijux_dna_stage_contract::ToolBindingV1;
use serde::{Deserialize, Serialize};

pub const BENCHMARK_CASE_MATRIX_SCHEMA_V1: &str = "bijux.benchmark.case_matrix.v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusSnapshotV1 {
    pub corpus_id: String,
    pub snapshot_sha256: String,
    pub sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseMeasurementPlanV1 {
    pub allocated_cpus: u32,
    pub warmup_repetitions: u32,
    pub measured_repetitions: u32,
    pub cache_policy: BenchmarkCachePolicy,
    pub performance_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseSciencePlanV1 {
    pub evidence_tier: BenchmarkEvidenceTier,
    pub profile_id: String,
    pub scientific_question: String,
    pub comparison_scope: String,
    pub accuracy_claim: String,
    pub interpretation: String,
    pub scientific_metrics: Vec<String>,
    pub comparison_metrics: Vec<BenchmarkComparisonMetricTolerance>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseDefinitionV1 {
    pub case_id: String,
    pub run_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus: BenchmarkCorpusSnapshotV1,
    pub binding: ToolBindingV1,
    pub interface: BenchmarkStageInterfaceSpec,
    pub measurement: BenchmarkCaseMeasurementPlanV1,
    pub science: BenchmarkCaseSciencePlanV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseMatrixV1 {
    pub schema_version: String,
    pub repository_commit: String,
    pub campaign_registry_sha256: String,
    pub interface_catalog_sha256: String,
    pub tool_binding_registry_sha256: String,
    pub cases: Vec<BenchmarkCaseDefinitionV1>,
}
