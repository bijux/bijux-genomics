use crate::foundation::{BijuxError, Result};

use super::super::PipelineId;

/// # Errors
/// Returns an error if the pipeline id is invalid.
pub fn parse_pipeline_id(value: &str) -> Result<PipelineId> {
    PipelineId::try_from(value)
}

/// # Errors
/// Returns an error if the pipeline id is invalid.
pub fn validate_pipeline_id(id: &PipelineId) -> Result<()> {
    validate_pipeline_id_str(id.as_str())
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
