use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationMetricsV1 {
    pub method: String,
    pub estimate: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    pub assumptions: Vec<String>,
}

impl ContaminationMetricsV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            method: "unknown".to_string(),
            estimate: 0.0,
            ci_low: 0.0,
            ci_high: 0.0,
            assumptions: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationReconciliationV1 {
    pub mt_fraction: Option<f64>,
    pub nuclear_fraction: Option<f64>,
    pub assessment: String,
}

impl ContaminationReconciliationV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            mt_fraction: None,
            nuclear_fraction: None,
            assessment: "unknown".to_string(),
        }
    }
}

impl Default for ContaminationReconciliationV1 {
    fn default() -> Self {
        Self::empty()
    }
}
