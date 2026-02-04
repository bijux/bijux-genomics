use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::RawFailure;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    UserError,
    DataError,
    ToolError,
    Bug,
    InfraError,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HintSeverity {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorHintV1 {
    pub id: String,
    pub category: ErrorCategory,
    pub severity: HintSeverity,
    pub message: String,
    pub suggested_action: String,
    pub docs_link_key: Option<String>,
}

#[derive(Debug, Error)]
#[error("{category:?}: {message}")]
pub struct CategorizedError {
    pub category: ErrorCategory,
    pub message: String,
}

impl CategorizedError {
    #[must_use]
    pub fn new(category: ErrorCategory, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
        }
    }
}

#[must_use]
pub fn remediation_hints_for_failure(raw: &RawFailure) -> Vec<ErrorHintV1> {
    let msg = raw.reason.to_lowercase();
    let mut hints = Vec::new();
    if msg.contains("adapter") || msg.contains("adapter preset") {
        hints.push(ErrorHintV1 {
            id: "adapter_preset_missing".to_string(),
            category: ErrorCategory::DataError,
            severity: HintSeverity::Medium,
            message: "Adapter preset missing or invalid".to_string(),
            suggested_action: "Configure a valid adapter preset or supply an adapter file"
                .to_string(),
            docs_link_key: Some("adapters".to_string()),
        });
    }
    if msg.contains("polyg") || msg.contains("poly-g") {
        hints.push(ErrorHintV1 {
            id: "polyg_artifact".to_string(),
            category: ErrorCategory::DataError,
            severity: HintSeverity::Low,
            message: "Poly-G artifact suspected".to_string(),
            suggested_action: "Enable illumina_twocolor or configure polyG filtering".to_string(),
            docs_link_key: Some("polyg".to_string()),
        });
    }
    if raw.stage == "fastq.screen" || msg.contains("contaminant") {
        hints.push(ErrorHintV1 {
            id: "contamination_screen".to_string(),
            category: ErrorCategory::DataError,
            severity: HintSeverity::Medium,
            message: "Potential contaminant signal detected".to_string(),
            suggested_action: "Review contaminant screen output and adjust contaminant bank"
                .to_string(),
            docs_link_key: Some("contamination".to_string()),
        });
    }
    if msg.contains("missing outputs") {
        hints.push(ErrorHintV1 {
            id: "missing_outputs".to_string(),
            category: ErrorCategory::ToolError,
            severity: HintSeverity::High,
            message: "Expected outputs missing".to_string(),
            suggested_action: "Check tool output paths, permissions, and working directory"
                .to_string(),
            docs_link_key: Some("outputs".to_string()),
        });
    }
    hints
}
