use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
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
        out.push(line.trim_start_matches("  - ").trim_matches('"').to_string());
    }
    out
}

fn inline_list(raw: &str, key: &str) -> Vec<String> {
    raw.lines()
        .find_map(|line| line.strip_prefix(&format!("{key}: [")))
        .and_then(|body| body.strip_suffix(']'))
        .map(|body| {
            body.split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|value| value.trim_matches('"').to_string())
                .collect()
        })
        .unwrap_or_default()
}

fn yaml_list(raw: &str, key: &str) -> Vec<String> {
    let block = block_list(raw, key);
    if block.is_empty() {
        inline_list(raw, key)
    } else {
        block
    }
}

fn quoted_scalar(raw: &str, key: &str) -> Result<String> {
    raw.lines()
        .find_map(|line| line.strip_prefix(&format!("{key}: ")))
        .map(|value| value.trim().trim_matches('"').to_string())
        .with_context(|| format!("{key} missing"))
}

fn stage_tool_expectations() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut out = BTreeMap::new();
    let stages_dir = workspace_root()?.join("domain/fastq/stages");
    for entry in std::fs::read_dir(&stages_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let stage_id = quoted_scalar(&raw, "stage_id")
            .with_context(|| format!("stage_id missing in {}", path.display()))?;
        let mut tools = yaml_list(&raw, "compatible_tools").into_iter().collect::<BTreeSet<_>>();
        tools.extend(yaml_list(&raw, "planned_out_of_scope"));
        out.insert(stage_id, tools);
    }
    Ok(out)
}

#[test]
fn tool_stage_bindings_are_admitted_or_explicitly_planned() -> Result<()> {
    let expected_by_stage = stage_tool_expectations()?;
    let tools_dir = workspace_root()?.join("domain/fastq/tools");
    for entry in std::fs::read_dir(&tools_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let tool_id = quoted_scalar(&raw, "tool_id")
            .with_context(|| format!("tool_id missing in {}", path.display()))?;
        let declared_stage_ids = yaml_list(&raw, "stage_ids")
            .into_iter()
            .chain(yaml_list(&raw, "planned_stage_ids"))
            .collect::<Vec<_>>();
        for stage_id in declared_stage_ids {
            if !stage_id.starts_with("fastq.") {
                continue;
            }
            let admitted = expected_by_stage
                .get(&stage_id)
                .with_context(|| format!("stage manifest missing for {stage_id}"))?;
            assert!(
                admitted.contains(&tool_id),
                "{tool_id} claims {stage_id}, but {stage_id} does not list it under compatible_tools or planned_out_of_scope"
            );
        }
    }
    Ok(())
}
