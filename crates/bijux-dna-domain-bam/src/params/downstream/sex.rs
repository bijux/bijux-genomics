use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::ExpectedSex;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SexEffectiveParams {
    #[serde(default)]
    pub expected_sex: Option<ExpectedSex>,
    pub method: String,
}
