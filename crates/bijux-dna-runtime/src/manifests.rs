use std::path::Path;

use anyhow::{anyhow, Context, Result};

use bijux_dna_core::contract::{StageSpec, ToolManifest, ToolRegistry};
use bijux_dna_infra::formats::parse_yaml;

/// # Errors
/// Returns an error if manifest files cannot be read or parsed.
pub fn load_manifests(domain_root: &Path) -> Result<ToolRegistry> {
    let mut registry = ToolRegistry::default();
    if !domain_root.exists() {
        return Err(anyhow!(
            "domain root {} does not exist",
            domain_root.display()
        ));
    }
    for entry in std::fs::read_dir(domain_root).context("read domain root")? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let stages_dir = path.join("stages");
        if stages_dir.exists() {
            for stage_entry in std::fs::read_dir(&stages_dir).context("read stages dir")? {
                let stage_entry = stage_entry?;
                let stage_path = stage_entry.path();
                if stage_path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with('_'))
                {
                    continue;
                }
                if stage_path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
                    continue;
                }
                let raw = std::fs::read_to_string(&stage_path)
                    .with_context(|| format!("read {}", stage_path.display()))?;
                let stage: StageSpec =
                    parse_yaml(&raw).with_context(|| format!("parse {}", stage_path.display()))?;
                registry.insert_stage(stage);
            }
        }
        let tools_dir = path.join("tools");
        if tools_dir.exists() {
            for tool_entry in std::fs::read_dir(&tools_dir).context("read tools dir")? {
                let tool_entry = tool_entry?;
                let tool_path = tool_entry.path();
                if tool_path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
                    continue;
                }
                let raw = std::fs::read_to_string(&tool_path)
                    .with_context(|| format!("read {}", tool_path.display()))?;
                let tool: ToolManifest =
                    parse_yaml(&raw).with_context(|| format!("parse {}", tool_path.display()))?;
                registry.insert_tool(tool);
            }
        }
    }
    registry.sort_tools_for_determinism();
    Ok(registry)
}
