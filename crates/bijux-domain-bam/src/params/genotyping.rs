use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GenotypingEffectiveParams {
    pub caller: String,
    #[serde(default)]
    pub min_posterior: Option<f64>,
    #[serde(default)]
    pub min_call_rate: Option<f64>,
}
