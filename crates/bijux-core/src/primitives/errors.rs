use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    PlanError,
    ToolError,
    ContractError,
    ParseError,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawFailure {
    pub stage: String,
    pub tool: String,
    pub reason: String,
    pub category: ErrorCategory,
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
