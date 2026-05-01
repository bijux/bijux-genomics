use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::ExpectedSex;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SexEffectiveParams {
    #[serde(default)]
    pub expected_sex: Option<ExpectedSex>,
    pub method: String,
    #[serde(default)]
    pub chromosome_system: Option<String>,
    #[serde(default)]
    pub minimum_y_sites: Option<u32>,
    #[serde(default = "default_refuse_without_context")]
    pub refuse_without_context: bool,
}

fn default_refuse_without_context() -> bool {
    true
}
