use crate::foundation::{BijuxError, Result};

use super::super::{ArtifactId, ProfileId};

fn validate_symbolic_id_str(kind: &str, id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(BijuxError::validation(format!("{kind} cannot be empty")));
    }
    if id.starts_with(['.', '-', '_']) || id.ends_with(['.', '-', '_']) {
        return Err(BijuxError::validation(format!("{kind} cannot start or end with a separator")));
    }
    let allowed =
        |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_';
    if !id.chars().all(allowed) {
        return Err(BijuxError::validation(format!("{kind} contains invalid characters")));
    }
    Ok(())
}

/// # Errors
/// Returns an error if the artifact id is invalid.
pub fn validate_artifact_id(id: &ArtifactId) -> Result<()> {
    validate_artifact_id_str(id.as_str())
}

/// # Errors
/// Returns an error if the artifact id is invalid.
pub fn validate_artifact_id_str(id: &str) -> Result<()> {
    validate_symbolic_id_str("artifact id", id)
}

/// # Errors
/// Returns an error if the profile id is invalid.
pub fn validate_profile_id(id: &ProfileId) -> Result<()> {
    validate_profile_id_str(id.as_str())
}

/// # Errors
/// Returns an error if the profile id is invalid.
pub fn validate_profile_id_str(id: &str) -> Result<()> {
    validate_symbolic_id_str("profile id", id)
}

#[cfg(test)]
mod tests {
    use super::{validate_artifact_id_str, validate_profile_id_str};

    #[test]
    fn symbolic_ids_reject_boundary_separators() {
        assert!(validate_artifact_id_str("_reads").is_err());
        assert!(validate_artifact_id_str("reads_").is_err());
        assert!(validate_profile_id_str("-reference_adna").is_err());
        assert!(validate_profile_id_str("reference_adna.").is_err());
    }

    #[test]
    fn symbolic_ids_accept_catalog_shapes() {
        assert!(validate_artifact_id_str("reads_out").is_ok());
        assert!(validate_profile_id_str("reference_adna").is_ok());
    }
}
