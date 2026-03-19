use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn stage_manifest_tools() -> Result<BTreeMap<String, Vec<String>>> {
    let stages_dir = workspace_root()?.join("domain/fastq/stages");
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(&stages_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;
        let stage_id = raw
            .lines()
            .find_map(|line| line.strip_prefix("stage_id: "))
            .map(|value| value.trim().trim_matches('"').to_string())
            .with_context(|| format!("stage_id missing in {}", path.display()))?;
        let compatible_tools = block_list(&raw, "compatible_tools");
        out.insert(stage_id, compatible_tools);
    }
    Ok(out)
}

fn indexed_stage_tools() -> Result<BTreeMap<String, Vec<String>>> {
    let raw = std::fs::read_to_string(workspace_root()?.join("domain/fastq/index.yaml"))
        .context("read domain/fastq/index.yaml")?;
    let mut out = BTreeMap::<String, Vec<String>>::new();
    let mut in_block = false;
    let mut current_stage = None::<String>;
    for line in raw.lines() {
        if line == "stage_tool_compatibility:" {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if !line.starts_with(' ') && line.contains(':') {
            break;
        }
        if let Some(stage) = line.strip_prefix("  ").and_then(|rest| rest.strip_suffix(':')) {
            let stage = stage.to_string();
            out.entry(stage.clone()).or_default();
            current_stage = Some(stage);
            continue;
        }
        if let Some(tool) = line.strip_prefix("  - ") {
            if let Some(stage) = &current_stage {
                out.entry(stage.clone()).or_default().push(tool.to_string());
            }
        }
    }
    Ok(out)
}

fn block_list(raw: &str, key: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_block = false;
    for line in raw.lines() {
        if line == format!("{key}:") {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if !line.starts_with("  - ") {
            break;
        }
        out.push(line.trim_start_matches("  - ").to_string());
    }
    out
}

#[test]
fn generated_index_stage_tool_compatibility_matches_stage_manifests() -> Result<()> {
    assert_eq!(
        indexed_stage_tools()?,
        stage_manifest_tools()?,
        "domain/fastq/index.yaml drifted from stage manifest compatible_tools"
    );
    Ok(())
}
