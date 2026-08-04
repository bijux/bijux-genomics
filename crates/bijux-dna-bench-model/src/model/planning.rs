//! Owner: bijux-dna-bench-model
//! Typed benchmark planning contracts for campaign and portfolio declarations.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkEvidenceTier {
    Qualification,
    Comparative,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkExecutionRuntime {
    Apptainer,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkExecutionScheduler {
    Slurm,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkCachePolicy {
    Cold,
    Warm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkComparisonMetricTolerance {
    pub id: String,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkStageCampaignRegistry {
    pub schema_version: String,
    pub campaigns: Vec<BenchmarkStageCampaignSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkMeasurementContract {
    pub warmup_repetitions: u32,
    pub measured_repetitions: u32,
    pub required_resource_metrics: Vec<String>,
    pub required_quality_dimensions: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkCorpusStatus {
    Missing,
    SourceDefined,
    Admitted,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkEvidenceScope {
    Qualification,
    Comparative,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkScientificProfile {
    pub id: String,
    pub required_corpus_roles: Vec<String>,
    pub minimum_real_samples: u32,
    pub accuracy_strategy: String,
    pub interchangeability: String,
    pub downstream_probe: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkProfileAssignment {
    pub profile_id: String,
    pub stage_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkPortfolio {
    pub schema_version: String,
    pub measurement: BenchmarkMeasurementContract,
    pub stage_runs: BTreeMap<String, String>,
    pub corpora: Vec<BenchmarkCorpusContract>,
    pub profiles: Vec<BenchmarkScientificProfile>,
    pub assignments: Vec<BenchmarkProfileAssignment>,
}
