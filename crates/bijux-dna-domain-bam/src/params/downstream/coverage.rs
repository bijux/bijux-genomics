use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::BedRegions;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageEffectiveParams {
    #[serde(default)]
    pub regions: Option<BedRegions>,
    pub depth_thresholds: Vec<u32>,
    #[serde(default = "default_coverage_regime_mode")]
    pub regime_mode: String,
}

fn default_coverage_regime_mode() -> String {
    "advisory_and_enforced".to_string()
}
