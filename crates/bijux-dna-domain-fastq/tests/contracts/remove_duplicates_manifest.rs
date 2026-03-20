use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn tool_manifest(tool_id: &str) -> Result<serde_json::Value> {
    let path = workspace_root()?.join(format!("domain/fastq/tools/{tool_id}.yaml"));
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    bijux_dna_infra::formats::parse_yaml(&raw).with_context(|| format!("parse {}", path.display()))
}

#[test]
fn admitted_deduplicate_tools_only_compare_with_stage_peers() -> Result<()> {
    let remove_duplicates_tools = ["fastuniq", "clumpify"];
    for tool_id in remove_duplicates_tools {
        let manifest = tool_manifest(tool_id)?;
        let comparable_with = manifest
            .get("comparability")
            .and_then(|value| value.get("comparable_with"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("comparability list missing for {tool_id}"))?;
        for comparable_tool in comparable_with {
            let comparable_tool = comparable_tool
                .as_str()
                .with_context(|| format!("non-string comparable_with entry for {tool_id}"))?;
            assert!(
                remove_duplicates_tools.contains(&comparable_tool),
                "deduplicate tool {tool_id} must not reference non-admitted remove_duplicates peer {comparable_tool}",
            );
        }
    }
    Ok(())
}
