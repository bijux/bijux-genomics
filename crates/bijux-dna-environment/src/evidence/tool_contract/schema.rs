use std::path::Path;

use schemars::schema_for;
use serde_json::Value;

use super::model::ContainerToolContract;

#[must_use]
pub fn tool_contract_schema() -> Value {
    let mut schema = serde_json::to_value(schema_for!(ContainerToolContract))
        .unwrap_or_else(|_| serde_json::json!({}));
    if let Some(object) = schema.as_object_mut() {
        object.insert(
            "oneOf".to_string(),
            serde_json::json!([
                {
                    "required": ["source_sha256"],
                    "not": {"required": ["pinned_commit"]}
                },
                {
                    "required": ["pinned_commit"],
                    "not": {"required": ["source_sha256"]}
                }
            ]),
        );
    }
    schema
}

/// Write the embedded tool-contract schema atomically.
///
/// # Errors
/// Returns an error when the destination cannot be written.
pub fn write_tool_contract_schema(path: &Path) -> anyhow::Result<()> {
    bijux_dna_infra::atomic_write_json(path, &tool_contract_schema())?;
    Ok(())
}
