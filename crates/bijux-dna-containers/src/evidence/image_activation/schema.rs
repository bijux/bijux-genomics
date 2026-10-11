use std::path::Path;

use anyhow::{Context, Result};
use schemars::schema_for;

use super::model::ImageActivationPlan;

#[must_use]
pub fn image_activation_plan_schema() -> serde_json::Value {
    serde_json::to_value(schema_for!(ImageActivationPlan)).unwrap_or_else(|_| serde_json::json!({}))
}

/// Write the typed image-activation plan schema atomically.
///
/// # Errors
/// Returns an error when schema serialization or publication fails.
pub fn write_image_activation_plan_schema(path: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_json(path, &image_activation_plan_schema())?;
    Ok(())
}

/// Compare one checked-in activation schema with the container owner's generated contract.
///
/// # Errors
/// Returns an error when the checked schema cannot be read or parsed.
pub fn image_activation_plan_schema_matches(path: &Path) -> Result<bool> {
    let bytes = std::fs::read(path).context("read image-activation plan schema")?;
    let checked: serde_json::Value =
        serde_json::from_slice(&bytes).context("parse image-activation plan schema")?;
    Ok(checked == image_activation_plan_schema())
}
