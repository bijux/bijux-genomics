//! Generated schema for selected container run evidence.

use std::path::Path;

use anyhow::Result;
use schemars::Schema;

use super::RunEvidenceIndex;

#[must_use]
pub fn run_evidence_index_schema() -> Schema {
    schemars::schema_for!(RunEvidenceIndex)
}

/// Write the selected run-evidence publication schema atomically.
///
/// # Errors
/// Returns an error when the schema cannot be written.
pub fn write_run_evidence_index_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &run_evidence_index_schema())?;
    Ok(())
}
