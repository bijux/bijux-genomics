use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct HaplogroupEffectiveParams {
    pub reference_panel: String,
    #[serde(default)]
    pub min_coverage: Option<f64>,
}
