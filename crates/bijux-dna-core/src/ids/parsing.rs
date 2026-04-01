use crate::foundation::{BijuxError, Result};

use super::{PipelineId, StageId, ToolId};

/// Canonical stage identifiers owned by bijux-dna-core.
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
        return Err(BijuxError::validation(
            "stage id contains invalid characters",
        ));
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
        return Err(BijuxError::validation(
            "tool id contains invalid characters",
        ));
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
