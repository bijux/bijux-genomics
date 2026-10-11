//! Scientific benchmark scoring policy inputs; repository discovery is caller-owned.

use serde::{Deserialize, Serialize};

pub const STAGE_SCORING_SCHEMA_VERSION: &str = "bijux.bench.local_stage_scoring.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageScoringDecisionMode {
    MultiToolRanking,
    SingleToolAcceptance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageScoringCorrectnessSignal {
    ScientificComparableMetrics,
    OutputContract,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageScoringApplicability {
    Required,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageScoringDirection {
    LowerIsBetter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageScoringMissingBehavior {
    ExcludeFromRanking,
    FallbackToDeclaredMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StageScoringFailureEffect {
    BlockRecommendation,
    MarkInsufficientEvidence,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageScoringWeights {
    pub correctness: f64,
    pub scientific_threshold: f64,
    pub runtime: f64,
    pub memory: f64,
    pub completion: f64,
    pub failure_class: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageScoringCorrectness {
    pub source_report_paths: Vec<String>,
    pub signal: StageScoringCorrectnessSignal,
    pub metric_ids: Vec<String>,
    pub minimum_present_result_rows: usize,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageScoringScientificThreshold {
    pub source_report_paths: Vec<String>,
    pub applicability: StageScoringApplicability,
    pub metric_ids: Vec<String>,
    pub gate: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageScoringRuntime {
    pub source_report_paths: Vec<String>,
    pub metric_id: String,
    pub direction: StageScoringDirection,
    pub missing_behavior: StageScoringMissingBehavior,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageScoringMemory {
    pub source_report_paths: Vec<String>,
    pub metric_id: String,
    pub fallback_metric_id: String,
    pub direction: StageScoringDirection,
    pub missing_behavior: StageScoringMissingBehavior,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageScoringCompletion {
    pub source_report_paths: Vec<String>,
    pub required_row_statuses: Vec<String>,
    pub required_execution_statuses: Vec<String>,
    pub non_scoring_execution_statuses: Vec<String>,
    pub required_success_ratio: f64,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageScoringFailureClass {
    pub source_report_paths: Vec<String>,
    pub blocking_class_ids: Vec<String>,
    pub insufficient_class_ids: Vec<String>,
    pub allowed_row_statuses: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageScoringFailureClassCatalogEntry {
    pub class_id: String,
    pub effect: StageScoringFailureEffect,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageScoringRow {
    pub domain: String,
    pub stage_id: String,
    pub readiness_kind: String,
    pub decision_mode: StageScoringDecisionMode,
    pub active_tool_count: usize,
    pub benchmark_ready_tool_count: usize,
    pub default_tool_id: String,
    pub benchmark_ready_tool_ids: Vec<String>,
    pub report_section_ids: Vec<String>,
    pub recommendation_gate: String,
    pub weights: StageScoringWeights,
    pub correctness: StageScoringCorrectness,
    pub scientific_threshold: StageScoringScientificThreshold,
    pub runtime: StageScoringRuntime,
    pub memory: StageScoringMemory,
    pub completion: StageScoringCompletion,
    pub failure_class: StageScoringFailureClass,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StageScoringConfig {
    pub schema_version: String,
    pub active_stage_catalog_path: String,
    pub scientific_threshold_config_path: String,
    pub full_benchmark_report_path: String,
    pub micro_benchmark_report_path: String,
    pub failure_class_report_path: String,
    pub failure_classes: Vec<StageScoringFailureClassCatalogEntry>,
    pub rows: Vec<StageScoringRow>,
}
