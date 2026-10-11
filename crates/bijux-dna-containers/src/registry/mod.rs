//! Container manifest projection from the generated tool registry.

use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Registry {
    #[serde(default)]
    tools: Vec<Tool>,
}

#[derive(Deserialize, Serialize)]
struct Tool {
    #[serde(rename(serialize = "tool_id"))]
    id: String,
    version: Option<String>,
    upstream: Option<String>,
    pinned_commit: Option<String>,
    #[serde(default)]
    runtimes: Vec<String>,
    dockerfile: Option<String>,
    apptainer_def: Option<String>,
}

/// Project container-capable tools into the stable container manifest schema.
///
/// Unknown registry fields remain owned by the registry's other consumers. The
/// projection preserves optional fields as JSON nulls and orders tools by ID.
///
/// # Errors
/// Returns an error when the registry cannot be read, is malformed, or has no tools.
pub fn registry_export_containers_value(registry_path: &Path) -> Result<serde_json::Value> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let registry: Registry =
        toml::from_str(&raw).with_context(|| format!("parse {}", registry_path.display()))?;
    if registry.tools.is_empty() {
        bail!("missing [[tools]] entries");
    }
    let mut containers = registry
        .tools
        .into_iter()
        .filter(|tool| {
            tool.runtimes.iter().any(|runtime| runtime == "docker" || runtime == "apptainer")
                && (tool.dockerfile.is_some() || tool.apptainer_def.is_some())
        })
        .collect::<Vec<_>>();
    containers.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(serde_json::json!({
        "schema_version": "bijux.container_manifest.v1",
        "containers": containers,
    }))
}
