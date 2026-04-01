use crate::foundation::{BijuxError, Result};

use super::super::ToolId;

/// # Errors
/// Returns an error if the tool id is invalid.
pub fn parse_tool_id(value: &str) -> Result<ToolId> {
    ToolId::try_from(value)
}

/// # Errors
/// Returns an error if the tool id is invalid.
pub fn validate_tool_id(id: &ToolId) -> Result<()> {
    validate_tool_id_str(id.as_str())
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
