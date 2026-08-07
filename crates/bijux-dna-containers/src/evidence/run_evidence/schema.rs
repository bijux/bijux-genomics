//! Generated schema for selected container run evidence.

use std::path::Path;

use anyhow::Result;
use schemars::Schema;

use super::{ResultLayoutPlan, RunEvidenceIndex, SupersededResultArchivePlan};

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

#[must_use]
pub fn result_layout_plan_schema() -> Schema {
    schemars::schema_for!(ResultLayoutPlan)
}

#[must_use]
pub fn superseded_result_archive_plan_schema() -> Schema {
    schemars::schema_for!(SupersededResultArchivePlan)
}

/// Write both result-movement schemas atomically.
///
/// # Errors
/// Returns an error when either schema cannot be written.
pub fn write_result_movement_schemas(layout_path: &Path, archive_path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(layout_path, &result_layout_plan_schema())?;
    bijux_dna_infra::atomic_write_json(archive_path, &superseded_result_archive_plan_schema())?;
    Ok(())
}
