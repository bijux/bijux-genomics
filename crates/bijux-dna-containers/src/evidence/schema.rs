use anyhow::Context;
use schemars::schema_for;
use serde_json::Value;
use std::path::Path;

use super::inventory::ImageInventoryScan;
use super::model::{ContainerImageInventoryPublication, RuntimeVerificationPublication};

#[must_use]
pub fn image_inventory_scan_schema() -> Value {
    let mut schema = serde_json::to_value(schema_for!(ImageInventoryScan))
        .unwrap_or_else(|_| serde_json::json!({}));
    if let Some(required) = schema.pointer_mut("/required").and_then(Value::as_array_mut) {
        let evidence_roots = Value::String("evidence_roots".to_string());
        if !required.contains(&evidence_roots) {
            required.push(evidence_roots);
        }
    }
    if let Some(evidence_roots) =
        schema.pointer_mut("/properties/evidence_roots").and_then(Value::as_object_mut)
    {
        evidence_roots.remove("default");
        evidence_roots.insert("minItems".to_string(), Value::from(1));
    }
    schema
}

#[must_use]
pub fn image_inventory_publication_schema() -> Value {
    serde_json::to_value(schema_for!(ContainerImageInventoryPublication))
        .unwrap_or_else(|_| serde_json::json!({}))
}

#[must_use]
pub fn runtime_verification_publication_schema() -> Value {
    let mut schema = serde_json::to_value(schema_for!(RuntimeVerificationPublication))
        .unwrap_or_else(|_| serde_json::json!({}));
    if let Some(required) =
        schema.pointer_mut("/$defs/ContainerRuntimeEvidence/required").and_then(Value::as_array_mut)
    {
        for field in ["identity", "manifest"] {
            let field = Value::String(field.to_string());
            if !required.contains(&field) {
                required.push(field);
            }
        }
    }
    if let Some(properties) = schema
        .pointer_mut("/$defs/ContainerRuntimeEvidence/properties")
        .and_then(Value::as_object_mut)
    {
        for field in ["identity", "manifest"] {
            properties.insert(
                field.to_string(),
                serde_json::json!({"$ref": "#/$defs/EvidenceSourceIdentity"}),
            );
        }
    }
    schema
}

/// Write the image-inventory publication schema atomically.
///
/// # Errors
/// Returns an error when the destination cannot be written.
pub fn write_image_inventory_publication_schema(path: &Path) -> anyhow::Result<()> {
    bijux_dna_infra::atomic_write_json(path, &image_inventory_publication_schema())?;
    Ok(())
}

/// Write the complete image-inventory scan schema atomically.
///
/// # Errors
/// Returns an error when the destination cannot be written.
pub fn write_image_inventory_scan_schema(path: &Path) -> anyhow::Result<()> {
    bijux_dna_infra::atomic_write_json(path, &image_inventory_scan_schema())?;
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

/// Compare the checked runtime-publication and tool-identity schemas with their owner contracts.
///
/// # Errors
/// Returns an error when either checked schema cannot be read or parsed.
pub fn runtime_verification_schemas_match(
    publication_path: &Path,
    identity_path: &Path,
) -> anyhow::Result<bool> {
    let publication_bytes =
        std::fs::read(publication_path).context("read runtime-verification publication schema")?;
    let publication: Value = serde_json::from_slice(&publication_bytes)
        .context("parse runtime-verification publication schema")?;
    let identity_bytes = std::fs::read(identity_path).context("read tool-image identity schema")?;
    let identity: Value =
        serde_json::from_slice(&identity_bytes).context("parse tool-image identity schema")?;
    Ok(publication == runtime_verification_publication_schema()
        && identity == super::tool_contract::tool_image_identity_evidence_schema())
}
