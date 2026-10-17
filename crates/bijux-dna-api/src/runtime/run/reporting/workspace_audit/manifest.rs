use super::Result;
use anyhow::Context;
use std::path::{Path, PathBuf};
use toml::Value;

/// A named member manifest declared by the supplied workspace root.
///
/// Stability: v1 (stable).
#[derive(Clone, Debug)]
pub struct WorkspaceCrateManifest {
    pub crate_name: String,
    pub manifest_path: PathBuf,
}

/// Read declared member manifests in crate-name order without changing their paths.
///
/// # Errors
/// Returns an error if the workspace or a declared member manifest cannot be read or parsed.
pub fn read_workspace_crate_manifests(cwd: &Path) -> Result<Vec<WorkspaceCrateManifest>> {
    let workspace_manifest_path = cwd.join("Cargo.toml");
    let manifest = std::fs::read_to_string(&workspace_manifest_path)
        .with_context(|| format!("read {}", workspace_manifest_path.display()))?;
    let value: Value = toml::from_str(&manifest)
        .with_context(|| format!("parse {}", workspace_manifest_path.display()))?;
    let members = value
        .get("workspace")
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .context("workspace.members missing from root Cargo.toml")?;

    let mut resolved = Vec::new();
    for member in members {
        let member = member.as_str().context("workspace.members must contain only string paths")?;
        let manifest_path = cwd.join(member).join("Cargo.toml");
        let crate_manifest = std::fs::read_to_string(&manifest_path)
            .with_context(|| format!("read {}", manifest_path.display()))?;
        let crate_value: Value = toml::from_str(&crate_manifest)
            .with_context(|| format!("parse {}", manifest_path.display()))?;
        let crate_name = crate_value
            .get("package")
            .and_then(|package| package.get("name"))
            .and_then(Value::as_str)
            .with_context(|| format!("package.name missing from {}", manifest_path.display()))?;
        resolved.push(WorkspaceCrateManifest { crate_name: crate_name.to_string(), manifest_path });
    }
    resolved.sort_by(|left, right| left.crate_name.cmp(&right.crate_name));
    Ok(resolved)
}
