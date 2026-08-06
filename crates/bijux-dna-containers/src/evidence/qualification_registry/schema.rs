//! Owner: bijux-dna-containers
//! Generated schemas for qualification-run registry evidence.

use std::path::Path;

use anyhow::Result;
use schemars::{schema_for, Schema};

use super::{QualificationRunIdAudit, QualificationRunRegistry};

#[must_use]
pub fn qualification_run_registry_schema() -> Schema {
    schema_for!(QualificationRunRegistry)
}

#[must_use]
pub fn qualification_run_id_audit_schema() -> Schema {
    schema_for!(QualificationRunIdAudit)
}

/// Write the qualification-run registry schema atomically.
///
/// # Errors
/// Returns an error when serialization or publication fails.
pub fn write_qualification_run_registry_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &qualification_run_registry_schema())?;
    Ok(())
}

/// Write the qualification-run ID audit schema atomically.
///
/// # Errors
/// Returns an error when serialization or publication fails.
pub fn write_qualification_run_id_audit_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &qualification_run_id_audit_schema())?;
    Ok(())
}
