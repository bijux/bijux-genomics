use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, BijuxError>;

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

#[derive(Debug, Error)]
pub enum BijuxError {
    #[error("contract error: {0}")]
    ContractError(String),
    #[error("validation error: {0}")]
    ValidationError(String),
    #[error("io error: {0}")]
    Io(String),
    #[error("serialization error: {0}")]
    Serde(String),
    #[error("tool error: {0}")]
    ToolError(String),
    #[error("policy error: {0}")]
    PolicyError(String),
}

impl BijuxError {
    #[must_use]
    pub fn contract(message: impl Into<String>) -> Self {
        Self::ContractError(message.into())
    }

    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self::ValidationError(message.into())
    }

    #[must_use]
    pub fn tool(message: impl Into<String>) -> Self {
        Self::ToolError(message.into())
    }

    #[must_use]
    pub fn policy(message: impl Into<String>) -> Self {
        Self::PolicyError(message.into())
    }
}

impl From<std::io::Error> for BijuxError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<serde_json::Error> for BijuxError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serde(err.to_string())
    }
}

impl From<regex::Error> for BijuxError {
    fn from(err: regex::Error) -> Self {
        Self::ValidationError(err.to_string())
    }
}
