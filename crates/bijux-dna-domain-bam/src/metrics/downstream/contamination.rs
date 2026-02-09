use anyhow::Context;
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
#[serde(rename_all = "snake_case")]
pub enum ContaminationInputScopeV1 {
    MtOnly,
    NuclearOnly,
    Both,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationRequiredInputsV1 {
    pub reference_panel: String,
    pub scope: ContaminationInputScopeV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationWarningV1 {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContaminationToolMetricsV1 {
    pub tool: String,
    pub estimate: f64,
    pub ci_low: f64,
    pub ci_high: f64,
    pub model_assumptions: Vec<String>,
    pub required_inputs: ContaminationRequiredInputsV1,
    pub warnings: Vec<ContaminationWarningV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SchmutziMetricsV1 {
    pub contamination: ContaminationToolMetricsV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct VerifyBamId2MetricsV1 {
    pub contamination: ContaminationToolMetricsV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContamMixMetricsV1 {
    pub contamination: ContaminationToolMetricsV1,
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

/// # Errors
/// Returns an error if the contamination JSON cannot be read or parsed.
pub fn parse_contamination_json(path: &std::path::Path) -> anyhow::Result<ContaminationMetricsV1> {
    let raw = std::fs::read_to_string(path).context("read contamination json")?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    Ok(ContaminationMetricsV1 {
        method: value
            .get("method")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        estimate: value
            .get("estimate")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        ci_low: value
            .get("ci_low")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        ci_high: value
            .get("ci_high")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0),
        assumptions: value
            .get("assumptions")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default(),
    })
}
