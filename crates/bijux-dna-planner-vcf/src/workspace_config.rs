use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

#[derive(Debug, serde::Deserialize)]
struct ParamRegistry {
    #[serde(default)]
    entries: Vec<ParamRegistryEntry>,
}

#[derive(Debug, serde::Deserialize)]
struct ParamRegistryEntry {
    stage_id: String,
    #[serde(default)]
    params: Vec<String>,
}

pub fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."))
}

/// # Errors
/// Returns an error if required tool declarations cannot be loaded from workspace config.
pub fn load_required_tools() -> Result<BTreeSet<String>> {
    let mut set = BTreeSet::<String>::new();
    let root = workspace_root();
    for rel in [
        "configs/ci/tools/required_tools_vcf.toml",
        "configs/ci/tools/required_tools_vcf_downstream.toml",
    ] {
        let raw = fs::read_to_string(root.join(rel))?;
        let parsed: toml::Value = toml::from_str(&raw)?;
        let arr = parsed
            .get("required_tools")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing required_tools in {rel}"))?;
        for item in arr {
            if let Some(tool_id) = item.as_str() {
                set.insert(tool_id.to_string());
            }
        }
    }
    Ok(set)
}

/// # Errors
/// Returns an error if tool registry entries cannot be loaded from workspace config.
pub fn load_registry_tools() -> Result<BTreeSet<String>> {
    let mut set = BTreeSet::<String>::new();
    let root = workspace_root();
    for rel in [
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let raw = fs::read_to_string(root.join(rel))?;
        let parsed: toml::Value = toml::from_str(&raw)?;
        let entries = parsed
            .get("tools")
            .and_then(toml::Value::as_array)
            .ok_or_else(|| anyhow!("missing tools in {rel}"))?;
        for entry in entries {
            if let Some(tool_id) = entry.get("id").and_then(toml::Value::as_str) {
                set.insert(tool_id.to_string());
            }
        }
    }
    Ok(set)
}

/// # Errors
/// Returns an error if the downstream param registry cannot be read or parsed.
pub fn allowed_params_for_stage(stage_id: &str) -> Result<BTreeSet<String>> {
    let path = workspace_root().join("configs/ci/params/param_registry_downstream.toml");
    let raw = fs::read_to_string(&path)?;
    let parsed: ParamRegistry = toml::from_str(&raw)?;
    let mut allow = BTreeSet::new();
    for entry in parsed.entries {
        if entry.stage_id == stage_id {
            for param_name in entry.params {
                allow.insert(param_name);
            }
        }
    }
    Ok(allow)
}
