use crate::foundation::{BijuxError, Result};

use super::super::StageId;

/// Canonical stage identifiers owned by bijux-dna-core.
/// # Errors
/// Returns an error if the stage id is invalid.
pub fn parse_stage_id(value: &str) -> Result<StageId> {
    StageId::try_from(value)
}

/// # Errors
/// Returns an error if the stage id is invalid.
pub fn validate_stage_id(id: &StageId) -> Result<()> {
    validate_stage_id_str(id.as_str())
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
    if id.split('.').any(str::is_empty) {
        return Err(BijuxError::validation("stage id segments cannot be empty"));
    }
    let allowed =
        |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_';
    if !id.chars().all(allowed) {
        return Err(BijuxError::validation("stage id contains invalid characters"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::id_catalog::{BAM_MARKDUP, FASTQ_PREFIX, FASTQ_TRIM};

    use super::validate_stage_id_str;

    #[test]
    fn stage_id_rejects_empty_segments() {
        assert!(validate_stage_id_str(FASTQ_PREFIX).is_err());
        assert!(validate_stage_id_str(".trim_reads").is_err());
        assert!(validate_stage_id_str(&format!("{FASTQ_PREFIX}.trim_reads")).is_err());
    }

    #[test]
    fn stage_id_accepts_canonical_segments() {
        assert!(validate_stage_id_str(FASTQ_TRIM).is_ok());
        assert!(validate_stage_id_str(BAM_MARKDUP).is_ok());
    }
}
