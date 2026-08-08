use std::path::Path;

use schemars::schema_for;
use serde_json::Value;

use super::model::SecurityProbePublication;

#[must_use]
pub fn security_probe_publication_schema() -> Value {
    serde_json::to_value(schema_for!(SecurityProbePublication))
        .unwrap_or_else(|_| serde_json::json!({}))
}

/// Write the raw security-probe publication schema atomically.
///
/// # Errors
/// Returns an error when the destination cannot be written.
pub fn write_security_probe_publication_schema(path: &Path) -> anyhow::Result<()> {
    bijux_dna_infra::atomic_write_json(path, &security_probe_publication_schema())?;
    Ok(())
}
