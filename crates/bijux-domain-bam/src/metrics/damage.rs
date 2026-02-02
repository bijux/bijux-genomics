use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DamageComparisonV1 {
    pub tool_a: String,
    pub tool_b: String,
    pub c_to_t_diff: f64,
    pub g_to_a_diff: f64,
    pub exceeds_threshold: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DamageMetricsV1 {
    pub c_to_t_5p: f64,
    pub g_to_a_3p: f64,
    pub pmd_score_histogram: Vec<(u8, u64)>,
}

impl DamageMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            c_to_t_5p: 0.0,
            g_to_a_3p: 0.0,
            pmd_score_histogram: Vec::new(),
        }
    }
}

#[must_use]
pub fn compare_damage_metrics(
    tool_a: &str,
    metrics_a: &DamageMetricsV1,
    tool_b: &str,
    metrics_b: &DamageMetricsV1,
    threshold: f64,
) -> DamageComparisonV1 {
    let c_to_t_diff = (metrics_a.c_to_t_5p - metrics_b.c_to_t_5p).abs();
    let g_to_a_diff = (metrics_a.g_to_a_3p - metrics_b.g_to_a_3p).abs();
    let exceeds_threshold = c_to_t_diff > threshold || g_to_a_diff > threshold;
    DamageComparisonV1 {
        tool_a: tool_a.to_string(),
        tool_b: tool_b.to_string(),
        c_to_t_diff,
        g_to_a_diff,
        exceeds_threshold,
    }
}
