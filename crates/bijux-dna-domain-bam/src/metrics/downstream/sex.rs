use anyhow::Context;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum SexConfidenceClass {
    Male,
    Female,
    Ambiguous,
    #[default]
    Insufficient,
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

/// # Errors
/// Returns an error if the sex JSON cannot be read or parsed.
pub fn parse_sex_json(path: &std::path::Path) -> anyhow::Result<SexInferenceV1> {
    let raw = std::fs::read_to_string(path).context("read sex json")?;
    let value: serde_json::Value = serde_json::from_str(&raw)?;
    let x_to_y = value
        .get("x_to_y_ratio")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let confidence = value
        .get("confidence")
        .and_then(serde_json::Value::as_f64)
        .unwrap_or(0.0);
    let sufficient = confidence >= 0.6 && x_to_y > 0.0;
    let classification = if !sufficient {
        SexConfidenceClass::Insufficient
    } else if x_to_y <= 0.6 {
        SexConfidenceClass::Male
    } else if x_to_y >= 1.5 {
        SexConfidenceClass::Female
    } else {
        SexConfidenceClass::Ambiguous
    };
    Ok(SexInferenceV1 {
        x_to_y_ratio: x_to_y,
        confidence,
        method: value
            .get("method")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
        classification,
        sufficient_data: sufficient,
    })
}
