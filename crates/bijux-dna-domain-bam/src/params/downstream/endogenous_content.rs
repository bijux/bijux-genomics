use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::BedRegions;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EndogenousContentEffectiveParams {
    #[serde(default)]
    pub regions: Option<BedRegions>,
    pub depth_thresholds: Vec<u32>,
    pub host_reference_scope: String,
    #[serde(default)]
    pub host_reference_digest: Option<String>,
    #[serde(default = "default_refuse_without_host_reference")]
    pub refuse_without_host_reference: bool,
}

fn default_refuse_without_host_reference() -> bool {
    true
}
