//! Owner: bijux-dna-bench-model
//! Typed benchmark planning contracts for campaign and portfolio declarations.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::{
    BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V1, BENCHMARK_STAGE_COVERAGE_SCHEMA_V1,
    BENCHMARK_STAGE_UNIVERSE_SCHEMA_V1,
};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkEvidenceTier {
    Qualification,
    Comparative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkExecutionRuntime {
    Apptainer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkExecutionScheduler {
    Slurm,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkCachePolicy {
    Cold,
    Warm,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkComparisonMetricTolerance {
    pub id: String,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageCampaignSpec {
    pub run_id: String,
    pub slug: String,
    pub domain: String,
    pub stage_id: String,
    pub evidence_tier: BenchmarkEvidenceTier,
    pub profile_id: String,
    pub corpus_ids: Vec<String>,
    pub scientific_question: String,
    pub comparison_scope: String,
    pub tool_ids: Vec<String>,
    pub corpus_catalog_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_catalog_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub database_catalog_ids: Vec<String>,
    pub input_size_keys: Vec<String>,
    pub result_artifact: String,
    pub runtime: BenchmarkExecutionRuntime,
    pub scheduler: BenchmarkExecutionScheduler,
    pub container_only: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub threads: Option<u32>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub threads_by_tool: BTreeMap<String, u32>,
    pub warmup_repetitions: u32,
    pub measured_repetitions: u32,
    pub cache_policy: BenchmarkCachePolicy,
    pub result_root_template: String,
    pub accuracy_claim: String,
    pub interpretation: String,
    pub performance_metrics: Vec<String>,
    pub scientific_metrics: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comparison_metrics: Vec<BenchmarkComparisonMetricTolerance>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageCampaignRegistry {
    #[schemars(schema_with = "benchmark_stage_campaigns_schema_version")]
    pub schema_version: String,
    pub campaigns: Vec<BenchmarkStageCampaignSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkMeasurementContract {
    pub warmup_repetitions: u32,
    pub measured_repetitions: u32,
    pub required_resource_metrics: Vec<String>,
    pub required_quality_dimensions: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkCorpusStatus {
    Missing,
    SourceDefined,
    Admitted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkEvidenceScope {
    Qualification,
    Comparative,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkCorpusContract {
    pub id: String,
    pub status: BenchmarkCorpusStatus,
    pub evidence_scope: BenchmarkEvidenceScope,
    pub real_data: bool,
    pub sample_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum_sample_count: Option<u32>,
    pub data_types: Vec<String>,
    pub roles: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_specs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub catalog_ids: Vec<String>,
    pub truth_strategy: String,
    pub scientific_rationale: String,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkScientificProfile {
    pub id: String,
    pub required_corpus_roles: Vec<String>,
    pub minimum_real_samples: u32,
    pub accuracy_strategy: String,
    pub interchangeability: String,
    pub downstream_probe: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkProfileAssignment {
    pub profile_id: String,
    pub stage_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPortfolio {
    #[schemars(schema_with = "benchmark_portfolio_schema_version")]
    pub schema_version: String,
    pub measurement: BenchmarkMeasurementContract,
    pub stage_runs: BTreeMap<String, String>,
    pub corpora: Vec<BenchmarkCorpusContract>,
    pub profiles: Vec<BenchmarkScientificProfile>,
    pub assignments: Vec<BenchmarkProfileAssignment>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkScientificReadinessStatus {
    Ready,
    Blocked,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStageInterfaceStatus {
    Closed,
    Missing,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkScientificReadinessSummary {
    pub stage_count: usize,
    pub reserved_run_count: usize,
    pub ready_count: usize,
    pub blocked_count: usize,
    pub corpus_count: usize,
    pub admitted_comparative_corpus_count: usize,
    pub qualification_campaign_count: usize,
    pub comparative_campaign_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkScientificReadinessDomain {
    pub stage_count: usize,
    pub ready_count: usize,
    pub blocked_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkScientificReadinessCorpus {
    pub id: String,
    pub status: BenchmarkCorpusStatus,
    pub evidence_scope: BenchmarkEvidenceScope,
    pub sample_count: u32,
    pub truth_strategy: String,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkScientificReadinessStage {
    pub stage_id: String,
    pub domain: String,
    pub profile_id: String,
    pub retained_tools: Vec<String>,
    pub available_comparative_samples: u32,
    #[serde(default)]
    pub campaign_run_id: Option<String>,
    pub reserved_run_id: String,
    #[serde(default)]
    pub campaign_tier: Option<BenchmarkEvidenceTier>,
    pub interface_status: BenchmarkStageInterfaceStatus,
    pub readiness: BenchmarkScientificReadinessStatus,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkScientificReadinessReport {
    #[schemars(schema_with = "benchmark_scientific_readiness_schema_version")]
    pub schema_version: String,
    pub summary: BenchmarkScientificReadinessSummary,
    pub domains: BTreeMap<String, BenchmarkScientificReadinessDomain>,
    pub measurement: BenchmarkMeasurementContract,
    pub corpora: Vec<BenchmarkScientificReadinessCorpus>,
    pub stages: Vec<BenchmarkScientificReadinessStage>,
}

impl BenchmarkScientificReadinessReport {
    #[must_use]
    pub fn v1(
        summary: BenchmarkScientificReadinessSummary,
        domains: BTreeMap<String, BenchmarkScientificReadinessDomain>,
        measurement: BenchmarkMeasurementContract,
        corpora: Vec<BenchmarkScientificReadinessCorpus>,
        stages: Vec<BenchmarkScientificReadinessStage>,
    ) -> Self {
        Self {
            schema_version: BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V1.to_string(),
            summary,
            domains,
            measurement,
            corpora,
            stages,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStageDefinitionStatus {
    Missing,
    Qualification,
    Comparative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStageExecutionStatus {
    NotDefined,
    NotAdmitted,
    Stale,
    Failed,
    Admitted,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageCoverageSummary {
    pub retained_stage_count: usize,
    pub defined_stage_count: usize,
    pub qualification_stage_count: usize,
    pub comparative_stage_count: usize,
    pub admitted_stage_count: usize,
    pub stale_stage_count: usize,
    pub failed_stage_count: usize,
    pub missing_stage_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageCoverageRow {
    pub stage_id: String,
    pub retained_tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub campaign_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_repository_commit: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub report_sha256: Option<String>,
    pub definition_status: BenchmarkStageDefinitionStatus,
    pub execution_status: BenchmarkStageExecutionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageCoverage {
    #[schemars(schema_with = "benchmark_stage_coverage_schema_version")]
    pub schema_version: String,
    pub summary: BenchmarkStageCoverageSummary,
    pub stages: Vec<BenchmarkStageCoverageRow>,
}

impl BenchmarkStageCoverage {
    #[must_use]
    pub fn v1(
        summary: BenchmarkStageCoverageSummary,
        stages: Vec<BenchmarkStageCoverageRow>,
    ) -> Self {
        Self { schema_version: BENCHMARK_STAGE_COVERAGE_SCHEMA_V1.to_string(), summary, stages }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkDisposition {
    BenchmarkRequired,
    QualificationOnly,
    OrchestrationOnly,
    TestOnly,
    Retired,
    Excluded,
}

impl BenchmarkDisposition {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BenchmarkRequired => "benchmark_required",
            Self::QualificationOnly => "qualification_only",
            Self::OrchestrationOnly => "orchestration_only",
            Self::TestOnly => "test_only",
            Self::Retired => "retired",
            Self::Excluded => "excluded",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageUniverseSummary {
    pub stage_count: usize,
    pub benchmark_required_count: usize,
    pub qualification_only_count: usize,
    pub orchestration_only_count: usize,
    pub test_only_count: usize,
    pub retired_count: usize,
    pub excluded_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageUniverseRow {
    pub stage_id: String,
    pub domain: String,
    pub status: String,
    pub benchmark_disposition: BenchmarkDisposition,
    pub benchmark_disposition_reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reserved_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageUniverse {
    #[schemars(schema_with = "benchmark_stage_universe_schema_version")]
    pub schema_version: String,
    pub summary: BenchmarkStageUniverseSummary,
    pub stages: Vec<BenchmarkStageUniverseRow>,
}

impl BenchmarkStageUniverse {
    #[must_use]
    pub fn v1(
        summary: BenchmarkStageUniverseSummary,
        stages: Vec<BenchmarkStageUniverseRow>,
    ) -> Self {
        Self { schema_version: BENCHMARK_STAGE_UNIVERSE_SCHEMA_V1.to_string(), summary, stages }
    }
}

#[must_use]
pub fn benchmark_stage_campaign_registry_schema() -> Schema {
    schemars::schema_for!(BenchmarkStageCampaignRegistry)
}

#[must_use]
pub fn benchmark_portfolio_schema() -> Schema {
    schemars::schema_for!(BenchmarkPortfolio)
}

#[must_use]
pub fn benchmark_scientific_readiness_schema() -> Schema {
    schemars::schema_for!(BenchmarkScientificReadinessReport)
}

fn benchmark_stage_campaigns_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_STAGE_CAMPAIGNS_SCHEMA_V1,
    })
}

fn benchmark_portfolio_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": crate::contract::BENCHMARK_PORTFOLIO_SCHEMA_V1,
    })
}

fn benchmark_scientific_readiness_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_SCIENTIFIC_READINESS_SCHEMA_V1,
    })
}

#[must_use]
pub fn benchmark_stage_coverage_schema() -> Schema {
    schemars::schema_for!(BenchmarkStageCoverage)
}

#[must_use]
pub fn benchmark_stage_universe_schema() -> Schema {
    schemars::schema_for!(BenchmarkStageUniverse)
}

fn benchmark_stage_coverage_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_STAGE_COVERAGE_SCHEMA_V1,
    })
}

fn benchmark_stage_universe_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": BENCHMARK_STAGE_UNIVERSE_SCHEMA_V1,
    })
}
