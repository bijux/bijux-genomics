use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FragmentLengthSummaryV1 {
    pub mean: f64,
    pub median: f64,
    pub p10: f64,
    pub p90: f64,
    pub short_fraction: f64,
}

impl FragmentLengthSummaryV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            mean: 0.0,
            median: 0.0,
            p10: 0.0,
            p90: 0.0,
            short_fraction: 0.0,
        }
    }
}
