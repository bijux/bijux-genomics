//! Owner: bijux-dna-bench-model
//! Persisted stage-profile data-fitness rubric.

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::contract::BENCHMARK_DATA_FITNESS_SCHEMA_V1;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataFitnessRubric {
    #[schemars(schema_with = "data_fitness_schema_version")]
    pub schema_version: String,
    pub defaults: BenchmarkDataFitnessDefaults,
    pub exception_policy: BenchmarkDataFitnessExceptionPolicy,
    pub profiles: Vec<BenchmarkDataFitnessProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataFitnessDefaults {
    pub minimum_real_samples: u32,
    pub minimum_cases_per_tool: u32,
    pub minimum_distinct_studies: u32,
    pub minimum_input_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataFitnessExceptionPolicy {
    pub required_fields: Vec<String>,
    pub review_authority: String,
    pub claim_restriction_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkDataFitnessProfile {
    pub profile_id: String,
    pub minimum_real_samples: u32,
    pub minimum_cases_per_tool: u32,
    pub minimum_distinct_studies: u32,
    pub minimum_input_bytes: u64,
    pub representativeness_strata: Vec<String>,
    pub required_layouts: Vec<String>,
    pub truth_requirement: BenchmarkTruthEvidenceRequirement,
    pub required_controls: Vec<String>,
    pub contamination_damage: BenchmarkStratificationRequirement,
    pub ancestry: BenchmarkStratificationRequirement,
    pub access: BenchmarkDataAccessRequirement,
    pub bias_assessments: Vec<String>,
    pub required_limitations: Vec<String>,
    pub allowed_claims: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scientific_exceptions: Vec<BenchmarkFitnessException>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkTruthEvidenceRequirement {
    CuratedTruth,
    MaskedTruth,
    OrthogonalExpectation,
    ObservationalPerformanceOnly,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkStratificationRequirement {
    Required,
    DeclareAndStratifyIfPresent,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
// These are independent persisted admission controls, not one state with four representations.
#[allow(clippy::struct_excessive_bools)]
pub struct BenchmarkDataAccessRequirement {
    pub stable_source_identity: bool,
    pub license_identity: bool,
    pub redistribution_classification: bool,
    pub checksum_before_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BenchmarkFitnessException {
    pub dimension: BenchmarkFitnessExceptionDimension,
    pub default_minimum: u64,
    pub permitted_minimum: u64,
    pub scientific_rationale: String,
    pub compensating_evidence: Vec<String>,
    pub allowed_claims: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BenchmarkFitnessExceptionDimension {
    MinimumRealSamples,
    MinimumCasesPerTool,
    MinimumDistinctStudies,
    MinimumInputBytes,
}

#[must_use]
pub fn benchmark_data_fitness_rubric_schema() -> Schema {
    schemars::schema_for!(BenchmarkDataFitnessRubric)
}

fn data_fitness_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": BENCHMARK_DATA_FITNESS_SCHEMA_V1})
}
