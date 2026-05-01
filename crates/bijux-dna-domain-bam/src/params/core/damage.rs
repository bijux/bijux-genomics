use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::UdgModel;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DamageEffectiveParams {
    pub udg_model: UdgModel,
    pub pmd_threshold_5p: f64,
    pub pmd_threshold_3p: f64,
    pub trim_5p: u8,
    pub trim_3p: u8,
    #[serde(default)]
    pub damage_tool_profile: Option<String>,
    #[serde(default = "default_evidence_only")]
    pub evidence_only: bool,
}

fn default_evidence_only() -> bool {
    true
}
