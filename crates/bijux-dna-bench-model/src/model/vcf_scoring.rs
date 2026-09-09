//! Owner: bijux-dna-bench-model
//! Typed VCF scoring observations and reports, independent of repository acquisition.

use crate::StageScoringConfig;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const VCF_TOOL_SCORES_SCHEMA_VERSION: &str = "bijux.bench.readiness.vcf_tool_scores.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VcfToolScoreStatus {
    Scored,
    InsufficientEvidence,
    Blocked,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfToolScoreRow {
    pub stage_id: String,
    pub tool_id: String,
    pub decision_mode: String,
    pub correctness_signal: String,
    pub result_ids: Vec<String>,
    pub report_row_ids: Vec<String>,
    pub corpus_ids: Vec<String>,
    pub report_sections: Vec<String>,
    pub row_statuses: Vec<String>,
    pub score_status: VcfToolScoreStatus,
    pub truth_correctness_score: Option<f64>,
    pub truth_correctness_basis: Option<String>,
    pub contract_correctness_score: Option<f64>,
    pub contract_correctness_basis: Option<String>,
    pub genotype_truth_metric_value: Option<f64>,
    pub genotype_truth_metric_basis: Option<String>,
    pub missingness_metric_value: Option<f64>,
    pub missingness_metric_basis: Option<String>,
    pub phasing_imputation_metric_value: Option<f64>,
    pub phasing_imputation_metric_basis: Option<String>,
    pub population_metric_value: Option<f64>,
    pub population_metric_basis: Option<String>,
    pub scientific_metric_ids: Vec<String>,
    pub scientific_metric_summary: Option<String>,
    pub runtime_seconds: Option<f64>,
    pub runtime_source: String,
    pub observed_memory_mb: Option<f64>,
    pub declared_memory_mb: Option<f64>,
    pub memory_source: String,
    pub failure_class: String,
    pub micro_execution_status: Option<String>,
    pub score_weight_coverage: f64,
    pub score_total: Option<f64>,
    pub evidence_paths: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VcfToolScoresReport {
    pub schema_version: &'static str,
    pub output_path: String,
    pub config_path: String,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub scored_row_count: usize,
    pub insufficient_evidence_row_count: usize,
    pub blocked_row_count: usize,
    pub failure_class_counts: BTreeMap<String, usize>,
    pub rows: Vec<VcfToolScoreRow>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VcfScoreExpectedRow {
    pub report_row_id: String,
    pub result_id: Option<String>,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub report_section: String,
    pub row_status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VcfScoreRuntimeRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub execution_status: String,
    pub elapsed_seconds: Option<f64>,
    pub runtime_source: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VcfScoreMemoryRow {
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub execution_status: String,
    pub declared_memory_mb: Option<f64>,
    pub observed_memory_mb: Option<f64>,
    pub memory_source: String,
}

#[derive(Debug, Clone, Default)]
pub struct VcfScoreEvidence {
    pub source_paths: BTreeSet<String>,
    pub truth_correctness_score: Option<f64>,
    pub truth_correctness_basis: Option<String>,
    pub contract_correctness_score: Option<f64>,
    pub contract_correctness_basis: Option<String>,
    pub genotype_truth_metric_value: Option<f64>,
    pub genotype_truth_metric_basis: Option<String>,
    pub missingness_metric_value: Option<f64>,
    pub missingness_metric_basis: Option<String>,
    pub phasing_imputation_metric_value: Option<f64>,
    pub phasing_imputation_metric_basis: Option<String>,
    pub population_metric_value: Option<f64>,
    pub population_metric_basis: Option<String>,
    pub scientific_metric_summary: Option<String>,
    pub runtime_seconds: Option<f64>,
    pub runtime_source: Option<String>,
}

/// Explicit policy, report projections and evidence supplied by repository adapters.
#[derive(Debug, Clone)]
pub struct VcfToolScoringInput {
    pub config: StageScoringConfig,
    pub expected_rows: Vec<VcfScoreExpectedRow>,
    pub runtime_rows: Vec<VcfScoreRuntimeRow>,
    pub memory_rows: Vec<VcfScoreMemoryRow>,
    pub evidence: BTreeMap<(String, String), VcfScoreEvidence>,
    pub external_artifact_bindings: BTreeSet<(String, String)>,
}
