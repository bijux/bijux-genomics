use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::ContaminationScope;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationEffectiveParams {
    pub reference_panels: Vec<String>,
    pub scope: ContaminationScope,
    #[serde(default)]
    pub prior: Option<f64>,
    pub sex_specific: bool,
    #[serde(default)]
    pub assumptions: Option<String>,
    #[serde(default)]
    pub required_reference_digest: Option<String>,
    #[serde(default)]
    pub chromosome_system: Option<String>,
    #[serde(default)]
    pub minimum_mean_coverage: Option<f64>,
    #[serde(default = "default_emit_confidence_caveats")]
    pub emit_confidence_caveats: bool,
}

fn default_emit_confidence_caveats() -> bool {
    true
}
