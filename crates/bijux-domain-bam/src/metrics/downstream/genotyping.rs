use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct GenotypingMetricsV1 {
    pub call_rate: f64,
    pub mean_posterior: f64,
    pub posterior_histogram: Vec<(u8, u64)>,
}

impl GenotypingMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            call_rate: 0.0,
            mean_posterior: 0.0,
            posterior_histogram: Vec::new(),
        }
    }
}
