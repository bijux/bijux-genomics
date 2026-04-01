use crate::foundation::{BijuxError, Result};

use super::super::{ArtifactId, ProfileId};

fn validate_symbolic_id_str(kind: &str, id: &str) -> Result<()> {
    if id.is_empty() {
        return Err(BijuxError::validation(format!("{kind} cannot be empty")));
    }
    let allowed =
        |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '-' || c == '_';
    if !id.chars().all(allowed) {
        return Err(BijuxError::validation(format!(
            "{kind} contains invalid characters"
        )));
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
