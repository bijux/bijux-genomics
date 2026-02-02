use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::common::{DuplicateAction, OpticalDuplicatePolicy, UmiPolicy};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MarkDupEffectiveParams {
    pub optical_duplicates: OpticalDuplicatePolicy,
    pub umi_policy: UmiPolicy,
    pub duplicate_action: DuplicateAction,
}
