use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageMetricsV1 {
    pub mean: f64,
    pub median: f64,
    pub breadth_1x: f64,
    pub breadth_3x: f64,
    pub breadth_5x: f64,
}

impl CoverageMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            breadth_1x: 0.0,
            breadth_3x: 0.0,
            breadth_5x: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageUniformityV1 {
    pub coefficient_of_variation: f64,
    pub dropout_fraction: f64,
}

impl CoverageUniformityV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            coefficient_of_variation: 0.0,
            dropout_fraction: 0.0,
        }
    }
}

impl Default for CoverageUniformityV1 {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectiveCoverageV1 {
    pub raw: f64,
    pub dedup: f64,
    pub pmd_filtered: f64,
}

impl EffectiveCoverageV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            raw: 0.0,
            dedup: 0.0,
            pmd_filtered: 0.0,
        }
    }
}

impl Default for EffectiveCoverageV1 {
    fn default() -> Self {
        Self::empty()
    }
}
