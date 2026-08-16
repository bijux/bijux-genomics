use std::path::Path;

use anyhow::{Context, Result};
use schemars::schema_for;

use super::model::ExternalArtifactAdmission;

#[must_use]
pub fn external_artifact_admission_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(ExternalArtifactAdmission))
        .unwrap_or_else(|_| serde_json::json!({}))
}

/// Write the external-artifact admission schema atomically.
///
/// # Errors
/// Returns an error when schema serialization or publication fails.
pub fn write_external_artifact_admission_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &external_artifact_admission_schema())?;
    Ok(())
}

/// Compare one checked-in admission schema with the container owner's generated contract.
///
/// # Errors
/// Returns an error when the checked schema cannot be read or parsed.
pub fn external_artifact_admission_schema_matches(path: &Path) -> Result<bool> {
    let bytes = std::fs::read(path).context("read external-artifact admission schema")?;
    let checked: serde_json::Value =
        serde_json::from_slice(&bytes).context("parse external-artifact admission schema")?;
    Ok(checked == external_artifact_admission_schema())
}
