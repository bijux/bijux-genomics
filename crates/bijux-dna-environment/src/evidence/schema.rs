use schemars::schema_for;
use serde_json::Value;
use std::path::Path;

use super::model::{ContainerImageInventoryPublication, RuntimeVerificationPublication};

#[must_use]
pub fn image_inventory_publication_schema() -> Value {
    serde_json::to_value(schema_for!(ContainerImageInventoryPublication))
        .unwrap_or_else(|_| serde_json::json!({}))
}

#[must_use]
pub fn runtime_verification_publication_schema() -> Value {
    serde_json::to_value(schema_for!(RuntimeVerificationPublication))
        .unwrap_or_else(|_| serde_json::json!({}))
}

/// Write the image-inventory publication schema atomically.
///
/// # Errors
/// Returns an error when the destination cannot be written.
pub fn write_image_inventory_publication_schema(path: &Path) -> anyhow::Result<()> {
    bijux_dna_infra::atomic_write_json(path, &image_inventory_publication_schema())?;
    Ok(())
}

/// Write the runtime-verification publication schema atomically.
///
/// # Errors
/// Returns an error when the destination cannot be written.
pub fn write_runtime_verification_publication_schema(path: &Path) -> anyhow::Result<()> {
    bijux_dna_infra::atomic_write_json(path, &runtime_verification_publication_schema())?;
    Ok(())
}
