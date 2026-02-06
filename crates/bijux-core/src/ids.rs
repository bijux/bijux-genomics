use std::borrow::Cow;

use serde::{Deserialize, Serialize};

use crate::primitives::{BijuxError, Result};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StageId(pub Cow<'static, str>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StepId(pub Cow<'static, str>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ToolId(pub Cow<'static, str>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ArtifactId(pub Cow<'static, str>);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProfileId(pub Cow<'static, str>);

pub type ToolVersion = String;
pub type ImageDigest = String;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PipelineId(pub Cow<'static, str>);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StageVersion(pub i32);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RunId(pub String);

impl StageId {
    #[must_use]
    pub const fn from_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(Cow::Owned(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl StepId {
    #[must_use]
    pub const fn from_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(Cow::Owned(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ToolId {
    #[must_use]
    pub const fn from_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(Cow::Owned(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ArtifactId {
    #[must_use]
    pub const fn from_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(Cow::Owned(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl ProfileId {
    #[must_use]
    pub const fn from_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(Cow::Owned(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl PipelineId {
    #[must_use]
    pub const fn from_static(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }

    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(Cow::Owned(value.into()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl RunId {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// # Errors
/// Returns an error if the stage id is invalid.
pub fn parse_stage_id(value: &str) -> Result<StageId> {
    StageId::try_from(value)
}

/// # Errors
/// Returns an error if the tool id is invalid.
pub fn parse_tool_id(value: &str) -> Result<ToolId> {
    ToolId::try_from(value)
}

/// # Errors
/// Returns an error if the pipeline id is invalid.
pub fn parse_pipeline_id(value: &str) -> Result<PipelineId> {
    PipelineId::try_from(value)
}

/// # Errors
/// Returns an error if the stage id is invalid.
pub fn validate_stage_id(id: &StageId) -> Result<()> {
    validate_stage_id_str(id.as_str())
}

/// # Errors
/// Returns an error if the tool id is invalid.
pub fn validate_tool_id(id: &ToolId) -> Result<()> {
    validate_tool_id_str(id.as_str())
}

/// # Errors
/// Returns an error if the pipeline id is invalid.
pub fn validate_pipeline_id(id: &PipelineId) -> Result<()> {
    validate_pipeline_id_str(id.as_str())
}

/// # Errors
/// Returns an error if the stage id is invalid.
pub fn validate_stage_id_str(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(BijuxError::validation("stage id cannot be empty"));
    }
    if !id.contains('.') {
        return Err(BijuxError::validation("stage id must contain '.'"));
    }
    let allowed =
        |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_';
    if !id.chars().all(allowed) {
        return Err(BijuxError::validation("stage id contains invalid characters"));
    }
    Ok(())
}

/// # Errors
/// Returns an error if the tool id is invalid.
pub fn validate_tool_id_str(id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(BijuxError::validation("tool id cannot be empty"));
    }
    let allowed =
        |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_';
    if !id.chars().all(allowed) {
        return Err(BijuxError::validation("tool id contains invalid characters"));
    }
    Ok(())
}

/// # Errors
/// Returns an error if the pipeline id is invalid.
pub fn validate_pipeline_id_str(id: &str) -> Result<()> {
    let parts: Vec<&str> = id.split("__").collect();
    if parts.len() != 3 {
        return Err(BijuxError::validation(
            "pipeline id must be <graph>__<flavor>__vN",
        ));
    }
    let graph = parts[0];
    let flavor = parts[1];
    let version = parts[2];
    if !graph.contains("-to-") {
        return Err(BijuxError::validation(
            "pipeline id graph must contain '-to-'",
        ));
    }
    if !version.starts_with('v') || version.len() < 2 || !version[1..].chars().all(char::is_numeric)
    {
        return Err(BijuxError::validation(
            "pipeline id version must be v<digits>",
        ));
    }
    let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_';
    if !graph.chars().all(allowed) || !flavor.chars().all(allowed) {
        return Err(BijuxError::validation(
            "pipeline id contains invalid characters",
        ));
    }
    Ok(())
}

impl std::fmt::Display for StageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for ToolId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for StepId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for ArtifactId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for ProfileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for PipelineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::fmt::Display for RunId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl TryFrom<&str> for StageId {
    type Error = BijuxError;

    fn try_from(value: &str) -> Result<Self> {
        validate_stage_id_str(value)?;
        Ok(Self::new(value))
    }
}

impl TryFrom<&str> for StepId {
    type Error = BijuxError;

    fn try_from(value: &str) -> Result<Self> {
        validate_stage_id_str(value)?;
        Ok(Self::new(value))
    }
}

impl TryFrom<&str> for ToolId {
    type Error = BijuxError;

    fn try_from(value: &str) -> Result<Self> {
        validate_tool_id_str(value)?;
        Ok(Self::new(value))
    }
}

impl TryFrom<&str> for ArtifactId {
    type Error = BijuxError;

    fn try_from(value: &str) -> Result<Self> {
        validate_tool_id_str(value)?;
        Ok(Self::new(value))
    }
}

impl TryFrom<&str> for ProfileId {
    type Error = BijuxError;

    fn try_from(value: &str) -> Result<Self> {
        validate_tool_id_str(value)?;
        Ok(Self::new(value))
    }
}

impl TryFrom<&str> for PipelineId {
    type Error = BijuxError;

    fn try_from(value: &str) -> Result<Self> {
        validate_pipeline_id_str(value)?;
        Ok(Self::new(value))
    }
}
