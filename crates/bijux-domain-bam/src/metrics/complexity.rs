use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ComplexityMetricsV1 {
    pub observed_reads: u64,
    pub projected_reads: Vec<(u64, u64)>,
}

impl ComplexityMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            observed_reads: 0,
            projected_reads: Vec::new(),
        }
    }
}
