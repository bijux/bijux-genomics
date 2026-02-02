use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::ContaminationScope;

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
}
