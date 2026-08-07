use std::path::Path;

use anyhow::Result;
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
