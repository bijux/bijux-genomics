use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComplexityEffectiveParams {
    pub min_reads: u64,
    pub projection_points: Vec<u64>,
}
