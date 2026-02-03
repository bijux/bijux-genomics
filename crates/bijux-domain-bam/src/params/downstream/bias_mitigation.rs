use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BiasMitigationEffectiveParams {
    pub gc_bias_correction: bool,
    pub map_bias_correction: bool,
}
