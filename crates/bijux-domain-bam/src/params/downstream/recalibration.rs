use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::BqsrMode;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecalibrationSkipCriteria {
    pub min_mean_coverage: f64,
    pub min_breadth_1x: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BqsrEffectiveParams {
    pub known_sites: Vec<String>,
    pub mode: BqsrMode,
    pub skip_criteria: RecalibrationSkipCriteria,
}
