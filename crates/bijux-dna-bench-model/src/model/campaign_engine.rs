//! Owner: bijux-dna-bench-model
//! Persisted registry-driven benchmark case, observation, and aggregation contracts.

use std::collections::BTreeMap;

use bijux_dna_stage_contract::ToolBindingV1;
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use super::{
    BenchmarkAttemptIdentity, BenchmarkCachePolicy, BenchmarkComparisonMetricTolerance,
    BenchmarkEvidenceTier, BenchmarkMetricDistribution, BenchmarkReportImage,
    BenchmarkReportInputLock, BenchmarkReportSlurmJob, BenchmarkScientificValue,
    BenchmarkStageCampaignSpec, BenchmarkStageInterfaceSpec,
};

use crate::contract::BENCHMARK_STAGE_NORMALIZATION_SCHEMA_V1;

pub const BENCHMARK_STAGE_NORMALIZATION_ARTIFACT: &str = "admission/stage-normalization.json";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageNormalizationUnitV1 {
    pub stage_instance_id: String,
    pub normalized_output: serde_json::Value,
    pub normalized_output_sha256: String,
    pub probe_id: String,
    pub binding_id: String,
    pub producer_stage_id: String,
    pub tool_id: String,
    pub passed: bool,
    pub checks: Vec<BenchmarkStageNormalizationCheckV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageNormalizationCheckV1 {
    pub assertion: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageNormalizationV1 {
    #[schemars(schema_with = "stage_normalization_schema_version")]
    pub schema_version: String,
    pub identity: BenchmarkAttemptIdentity,
    pub normalized_at_utc: String,
    pub stage_plan_manifest_sha256: String,
    pub units: Vec<BenchmarkStageNormalizationUnitV1>,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusSnapshotV1 {
    pub corpus_id: String,
    pub snapshot_sha256: String,
    pub sample_ids: Vec<String>,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord,
)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkDataSnapshotRoleV1 {
    Corpus,
    Reference,
    Database,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataArtifactSnapshotV1 {
    pub path: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataSnapshotV1 {
    pub catalog_id: String,
    pub role: BenchmarkDataSnapshotRoleV1,
    pub catalog_sha256: String,
    pub sample_ids: Vec<String>,
    #[serde(default)]
    pub files: Vec<BenchmarkDataArtifactSnapshotV1>,
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
pub struct BenchmarkCaseMatrixV2 {
    #[schemars(schema_with = "case_matrix_v2_schema_version")]
    pub schema_version: String,
    pub repository_commit: String,
    pub campaign_registry_sha256: String,
    pub interface_catalog_sha256: String,
    pub tool_binding_registry_sha256: String,
    pub data_snapshots: Vec<BenchmarkDataSnapshotV1>,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseExecutionEvidenceV1 {
    pub image: BenchmarkReportImage,
    pub slurm: BenchmarkReportSlurmJob,
    pub source_evidence_sha256: String,
    pub input_locks: Vec<BenchmarkReportInputLock>,
    pub input_bytes: BTreeMap<String, u64>,
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
pub struct BenchmarkAdmittedCaseObservationV2 {
    #[schemars(schema_with = "admitted_case_observation_v2_schema_version")]
    pub schema_version: String,
    pub case_id: String,
    pub repository_commit: String,
    pub admission_sha256: String,
    pub input_lock_sha256: String,
    pub parameter_lock_sha256: String,
    pub execution: BenchmarkCaseExecutionEvidenceV1,
    pub repetitions: Vec<BenchmarkCaseRepetitionObservationV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkAdmittedCaseObservationSetV1 {
    #[schemars(schema_with = "admitted_case_observation_set_schema_version")]
    pub schema_version: String,
    pub repository_commit: String,
    pub case_matrix_sha256: String,
    pub observations: Vec<BenchmarkAdmittedCaseObservationV2>,
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

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseAggregationV2 {
    #[schemars(schema_with = "case_aggregation_v2_schema_version")]
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
    pub execution: BenchmarkCaseExecutionEvidenceV1,
    pub repetition_count: u32,
    pub performance: BTreeMap<String, BenchmarkMetricDistribution>,
    pub scientific: BTreeMap<String, Vec<BenchmarkScientificValue>>,
    pub normalized_result_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCaseAggregationSetV1 {
    #[schemars(schema_with = "case_aggregation_set_schema_version")]
    pub schema_version: String,
    pub repository_commit: String,
    pub case_matrix_sha256: String,
    pub observation_set_sha256: String,
    pub aggregations: Vec<BenchmarkCaseAggregationV2>,
}

#[must_use]
pub fn benchmark_case_matrix_schema() -> Schema {
    schemars::schema_for!(BenchmarkCaseMatrixV2)
}

#[must_use]
pub fn benchmark_case_matrix_v1_schema() -> Schema {
    schemars::schema_for!(BenchmarkCaseMatrixV1)
}

#[must_use]
pub fn benchmark_stage_normalization_schema() -> Schema {
    schemars::schema_for!(BenchmarkStageNormalizationV1)
}

#[must_use]
pub fn benchmark_admitted_case_observation_schema() -> Schema {
    schemars::schema_for!(BenchmarkAdmittedCaseObservationV1)
}

#[must_use]
pub fn benchmark_admitted_case_observation_v2_schema() -> Schema {
    schemars::schema_for!(BenchmarkAdmittedCaseObservationV2)
}

#[must_use]
pub fn benchmark_admitted_case_observation_set_schema() -> Schema {
    schemars::schema_for!(BenchmarkAdmittedCaseObservationSetV1)
}

#[must_use]
pub fn benchmark_case_aggregation_schema() -> Schema {
    schemars::schema_for!(BenchmarkCaseAggregationV1)
}

#[must_use]
pub fn benchmark_case_aggregation_v2_schema() -> Schema {
    schemars::schema_for!(BenchmarkCaseAggregationV2)
}

#[must_use]
pub fn benchmark_case_aggregation_set_schema() -> Schema {
    schemars::schema_for!(BenchmarkCaseAggregationSetV1)
}

fn case_matrix_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_CASE_MATRIX_SCHEMA_V1,
    })
}

fn case_matrix_v2_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_CASE_MATRIX_SCHEMA_V2,
    })
}

fn stage_normalization_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_STAGE_NORMALIZATION_SCHEMA_V1,
    })
}

fn admitted_case_observation_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V1,
    })
}

fn admitted_case_observation_v2_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_ADMITTED_CASE_OBSERVATION_SCHEMA_V2,
    })
}

fn admitted_case_observation_set_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_ADMITTED_CASE_OBSERVATION_SET_SCHEMA_V1,
    })
}

fn case_aggregation_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_CASE_AGGREGATION_SCHEMA_V1,
    })
}

fn case_aggregation_v2_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_CASE_AGGREGATION_SCHEMA_V2,
    })
}

fn case_aggregation_set_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_CASE_AGGREGATION_SET_SCHEMA_V1,
    })
}
