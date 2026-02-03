use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct KinshipEffectiveParams {
    pub reference_panel: String,
    pub min_overlap_snps: u32,
}
