use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
