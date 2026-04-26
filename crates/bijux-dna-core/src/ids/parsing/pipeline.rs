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
        return Err(BijuxError::validation("pipeline id must be <graph>__<flavor>__vN"));
    }
    let graph = parts[0];
    let flavor = parts[1];
    let version = parts[2];
    if graph.is_empty() || flavor.is_empty() {
        return Err(BijuxError::validation("pipeline id graph and flavor cannot be empty"));
    }
    if !graph.contains("-to-") {
        return Err(BijuxError::validation("pipeline id graph must contain '-to-'"));
    }
    if !version.starts_with('v')
        || version.len() < 2
        || !version[1..].chars().all(|value| value.is_ascii_digit())
    {
        return Err(BijuxError::validation("pipeline id version must be v<digits>"));
    }
    let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_';
    if !graph.chars().all(allowed) || !flavor.chars().all(allowed) {
        return Err(BijuxError::validation("pipeline id contains invalid characters"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_pipeline_id_str;

    #[test]
    fn pipeline_id_rejects_empty_graph_or_flavor() {
        assert!(validate_pipeline_id_str("__default__v1").is_err());
        assert!(validate_pipeline_id_str("fastq-to-fastq____v1").is_err());
    }

    #[test]
    fn pipeline_id_version_requires_ascii_digits() {
        assert!(validate_pipeline_id_str("fastq-to-fastq__default__v١").is_err());
        assert!(validate_pipeline_id_str("fastq-to-fastq__default__v1").is_ok());
    }
}
