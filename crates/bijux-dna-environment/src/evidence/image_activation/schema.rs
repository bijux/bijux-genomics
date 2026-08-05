use std::path::Path;

use anyhow::Result;
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
