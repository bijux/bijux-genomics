//! Canonical BAM metrics schema v1.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AlignmentCountsV1 {
    pub total: u64,
    pub primary: u64,
    pub mapped: u64,
    pub proper_pair: u64,
    pub duplicates: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FragmentLengthSummaryV1 {
    pub mean: f64,
    pub median: f64,
    pub p10: f64,
    pub p90: f64,
    pub short_fraction: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MapqSummaryV1 {
    pub mean: f64,
    pub median: f64,
    pub p10: f64,
    pub p90: f64,
    pub histogram: Vec<(u8, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageMetricsV1 {
    pub mean: f64,
    pub median: f64,
    pub breadth_1x: f64,
    pub breadth_3x: f64,
    pub breadth_5x: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DamageMetricsV1 {
    pub c_to_t_5p: f64,
    pub g_to_a_3p: f64,
    pub pmd_score_histogram: Vec<(u8, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationMetricsV1 {
    pub method: String,
    pub estimate: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    pub assumptions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SexInferenceV1 {
    pub x_to_y_ratio: f64,
    pub confidence: f64,
    pub method: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComplexityMetricsV1 {
    pub observed_reads: u64,
    pub projected_reads: Vec<(u64, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GenotypingMetricsV1 {
    pub call_rate: f64,
    pub mean_posterior: f64,
    pub posterior_histogram: Vec<(u8, u64)>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BamMetricsV1 {
    pub schema_version: String,
    pub alignment: AlignmentCountsV1,
    pub fragment_length: FragmentLengthSummaryV1,
    pub mapq: MapqSummaryV1,
    pub coverage: CoverageMetricsV1,
    pub damage: DamageMetricsV1,
    pub contamination: ContaminationMetricsV1,
    pub sex: SexInferenceV1,
    pub complexity: ComplexityMetricsV1,
    pub genotyping: GenotypingMetricsV1,
}
