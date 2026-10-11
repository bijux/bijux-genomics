use std::path::Path;

use anyhow::Result;
use schemars::schema_for;

use super::model::{ImageStoragePlan, ImageStorageReport};

#[must_use]
pub fn image_storage_plan_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(ImageStoragePlan)).unwrap_or_else(|_| serde_json::json!({}))
}

#[must_use]
pub fn image_storage_report_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(ImageStorageReport)).unwrap_or_else(|_| serde_json::json!({}))
}

/// Write the typed image-storage plan schema atomically.
///
/// # Errors
/// Returns an error when schema serialization or publication fails.
pub fn write_image_storage_plan_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &image_storage_plan_schema())?;
    Ok(())
}

/// Write the typed image-storage report schema atomically.
///
/// # Errors
/// Returns an error when schema serialization or publication fails.
pub fn write_image_storage_report_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &image_storage_report_schema())?;
    Ok(())
}
