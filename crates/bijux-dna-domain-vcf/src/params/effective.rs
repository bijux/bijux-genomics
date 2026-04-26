use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{VcfCallParams, VcfFilterParams, VcfStatsParams};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(tag = "kind", content = "value")]
pub enum VcfEffectiveParams {
    Call(VcfCallParams),
    Filter(VcfFilterParams),
    Stats(VcfStatsParams),
}
