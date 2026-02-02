use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::artifacts::BedRegions;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CoverageEffectiveParams {
    #[serde(default)]
    pub regions: Option<BedRegions>,
    pub depth_thresholds: Vec<u32>,
}
