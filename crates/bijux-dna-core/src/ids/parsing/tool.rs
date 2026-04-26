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
    if id.starts_with(['.', '-', '_']) || id.ends_with(['.', '-', '_']) {
        return Err(BijuxError::validation("tool id cannot start or end with a separator"));
    }
    let allowed =
        |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_';
    if !id.chars().all(allowed) {
        return Err(BijuxError::validation("tool id contains invalid characters"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_tool_id_str;

    #[test]
    fn tool_id_rejects_boundary_separators() {
        assert!(validate_tool_id_str("_fastp").is_err());
        assert!(validate_tool_id_str("fastp_").is_err());
        assert!(validate_tool_id_str("-fastp").is_err());
        assert!(validate_tool_id_str("fastp.").is_err());
    }

    #[test]
    fn tool_id_accepts_catalog_shapes() {
        assert!(validate_tool_id_str("fastp").is_ok());
        assert!(validate_tool_id_str("seqkit_stats").is_ok());
        assert!(validate_tool_id_str("verifybamid2").is_ok());
    }
}
