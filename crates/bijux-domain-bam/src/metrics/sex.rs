use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SexConfidenceClass {
    Male,
    Female,
    Ambiguous,
    Insufficient,
}

impl Default for SexConfidenceClass {
    fn default() -> Self {
        Self::Insufficient
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SexInferenceV1 {
    pub x_to_y_ratio: f64,
    pub confidence: f64,
    pub method: String,
    #[serde(default)]
    pub classification: SexConfidenceClass,
    #[serde(default)]
    pub sufficient_data: bool,
}

impl SexInferenceV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            x_to_y_ratio: 0.0,
            confidence: 0.0,
            method: "unknown".to_string(),
            classification: SexConfidenceClass::Insufficient,
            sufficient_data: false,
        }
    }
}
