use std::path::Path;

use anyhow::{Context, Result};
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

/// Compare checked image-storage schemas with their container-owner contracts.
///
/// # Errors
/// Returns an error when either checked schema cannot be read or parsed.
pub fn image_storage_schemas_match(plan_path: &Path, report_path: &Path) -> Result<bool> {
    let plan_bytes = std::fs::read(plan_path).context("read image-storage plan schema")?;
    let plan: serde_json::Value =
        serde_json::from_slice(&plan_bytes).context("parse image-storage plan schema")?;
    let report_bytes = std::fs::read(report_path).context("read image-storage report schema")?;
    let report: serde_json::Value =
        serde_json::from_slice(&report_bytes).context("parse image-storage report schema")?;
    Ok(plan == image_storage_plan_schema() && report == image_storage_report_schema())
}
