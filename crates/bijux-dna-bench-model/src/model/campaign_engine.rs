//! Owner: bijux-dna-bench-model
//! Persisted registry-driven benchmark case, observation, and aggregation contracts.

use std::collections::BTreeMap;

use bijux_dna_stage_contract::ToolBindingV1;
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::{
    BenchmarkCachePolicy, BenchmarkComparisonMetricTolerance, BenchmarkEvidenceTier,
    BenchmarkMetricDistribution, BenchmarkScientificValue, BenchmarkStageCampaignSpec,
    BenchmarkStageInterfaceSpec,
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusSnapshotV1 {
    pub corpus_id: String,
    pub snapshot_sha256: String,
    pub sample_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseMeasurementPlanV1 {
    pub allocated_cpus: u32,
    pub warmup_repetitions: u32,
    pub measured_repetitions: u32,
    pub cache_policy: BenchmarkCachePolicy,
    pub performance_metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseDefinitionV1 {
    pub case_id: String,
    pub run_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus: BenchmarkCorpusSnapshotV1,
    pub campaign: BenchmarkStageCampaignSpec,
    pub binding: ToolBindingV1,
    pub interface: BenchmarkStageInterfaceSpec,
    pub measurement: BenchmarkCaseMeasurementPlanV1,
    pub science: BenchmarkCaseSciencePlanV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseMatrixV1 {
    #[schemars(schema_with = "case_matrix_schema_version")]
    pub schema_version: String,
    pub repository_commit: String,
    pub campaign_registry_sha256: String,
    pub interface_catalog_sha256: String,
    pub tool_binding_registry_sha256: String,
    pub cases: Vec<BenchmarkCaseDefinitionV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseRepetitionObservationV1 {
    pub repetition: u32,
    pub allocated_cpus: u32,
    pub performance: BTreeMap<String, f64>,
    pub scientific: BTreeMap<String, BenchmarkScientificValue>,
    pub normalized_result_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAdmittedCaseObservationV1 {
    #[schemars(schema_with = "admitted_case_observation_schema_version")]
    pub schema_version: String,
    pub case_id: String,
    pub repository_commit: String,
    pub admission_sha256: String,
    pub input_lock_sha256: String,
    pub parameter_lock_sha256: String,
    pub repetitions: Vec<BenchmarkCaseRepetitionObservationV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseAggregationV1 {
    #[schemars(schema_with = "case_aggregation_schema_version")]
    pub schema_version: String,
    pub case_id: String,
    pub run_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus: BenchmarkCorpusSnapshotV1,
    pub repository_commit: String,
    pub admission_sha256: String,
    pub input_lock_sha256: String,
    pub parameter_lock_sha256: String,
    pub repetition_count: u32,
    pub performance: BTreeMap<String, BenchmarkMetricDistribution>,
    pub scientific: BTreeMap<String, Vec<BenchmarkScientificValue>>,
    pub normalized_result_sha256: String,
}

#[must_use]
pub fn benchmark_case_matrix_schema() -> Schema {
    schemars::schema_for!(BenchmarkCaseMatrixV1)
}

#[must_use]
pub fn benchmark_admitted_case_observation_schema() -> Schema {
    schemars::schema_for!(BenchmarkAdmittedCaseObservationV1)
}

#[must_use]
pub fn benchmark_case_aggregation_schema() -> Schema {
    schemars::schema_for!(BenchmarkCaseAggregationV1)
}

fn case_matrix_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_CASE_MATRIX_SCHEMA_V1,
    })
}

fn admitted_case_observation_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1,
    })
}

fn case_aggregation_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_CASE_AGGREGATION_SCHEMA_V1,
    })
}
