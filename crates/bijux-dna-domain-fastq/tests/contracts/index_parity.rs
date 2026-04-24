use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;
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
        let raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
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
        if let Some((stage, inline_value)) =
            line.strip_prefix("  ").and_then(|rest| rest.split_once(':'))
        {
            let stage = stage.to_string();
            out.entry(stage.clone()).or_default();
            current_stage = Some(stage);
            if inline_value.trim() == "[]" {
                current_stage = None;
            }
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

fn indexed_stage_tool_integration() -> Result<BTreeMap<String, BTreeMap<String, String>>> {
    let raw = std::fs::read_to_string(workspace_root()?.join("domain/fastq/index.yaml"))
        .context("read domain/fastq/index.yaml")?;
    let mut out = BTreeMap::<String, BTreeMap<String, String>>::new();
    let mut in_block = false;
    let mut current_stage = None::<String>;
    for line in raw.lines() {
        if line == "stage_tool_integration:" {
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
        if let Some((tool_id, level)) =
            line.strip_prefix("    ").and_then(|rest| rest.split_once(':'))
        {
            if let Some(stage) = &current_stage {
                out.entry(stage.clone())
                    .or_default()
                    .insert(tool_id.trim().to_string(), level.trim().to_string());
            }
        }
    }
    Ok(out)
}

fn indexed_reference_index_compatibility() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let raw = std::fs::read_to_string(workspace_root()?.join("domain/fastq/index.yaml"))
        .context("read domain/fastq/index.yaml")?;
    let mut out = BTreeMap::<String, BTreeSet<String>>::new();
    let mut in_block = false;
    let mut current_tool = None::<String>;
    for line in raw.lines() {
        if line == "reference_index_compatibility:" {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if !line.starts_with(' ') && line.contains(':') {
            break;
        }
        if let Some(tool_id) = line.strip_prefix("  ").and_then(|rest| rest.strip_suffix(':')) {
            let tool_id = tool_id.to_string();
            out.entry(tool_id.clone()).or_default();
            current_tool = Some(tool_id);
            continue;
        }
        if let Some(backend) = line.strip_prefix("  - ") {
            if let Some(tool_id) = &current_tool {
                out.entry(tool_id.clone()).or_default().insert(backend.to_string());
            }
        }
    }
    Ok(out)
}

fn indexed_tool_ids() -> Result<BTreeSet<String>> {
    let raw = std::fs::read_to_string(workspace_root()?.join("domain/fastq/index.yaml"))
        .context("read domain/fastq/index.yaml")?;
    Ok(block_list(&raw, "tool_ids").into_iter().collect())
}

fn indexed_governed_stage_ids() -> Result<BTreeSet<String>> {
    let raw = std::fs::read_to_string(workspace_root()?.join("domain/fastq/index.yaml"))
        .context("read domain/fastq/index.yaml")?;
    Ok(block_list(&raw, "governed_stage_ids").into_iter().collect())
}

fn indexed_governed_tool_ids() -> Result<BTreeSet<String>> {
    let raw = std::fs::read_to_string(workspace_root()?.join("domain/fastq/index.yaml"))
        .context("read domain/fastq/index.yaml")?;
    Ok(block_list(&raw, "governed_tool_ids").into_iter().collect())
}

fn indexed_active_defaults() -> Result<BTreeMap<String, String>> {
    let raw = std::fs::read_to_string(workspace_root()?.join("domain/fastq/index.yaml"))
        .context("read domain/fastq/index.yaml")?;
    let mut out = BTreeMap::new();
    let mut in_block = false;
    for line in raw.lines() {
        if line == "active_defaults:" {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if !line.starts_with("  ") {
            break;
        }
        if let Some((stage_id, tool_id)) = line.trim().split_once(':') {
            out.insert(stage_id.trim().to_string(), tool_id.trim().to_string());
        }
    }
    Ok(out)
}

fn manifest_stage_statuses() -> Result<BTreeMap<String, String>> {
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
        let raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let stage_id = raw
            .lines()
            .find_map(|line| line.strip_prefix("stage_id: "))
            .map(|value| value.trim().trim_matches('"').to_string())
            .with_context(|| format!("stage_id missing in {}", path.display()))?;
        let status = raw
            .lines()
            .find_map(|line| line.strip_prefix("status: "))
            .map(|value| value.trim().trim_matches('"').to_string())
            .with_context(|| format!("status missing in {}", path.display()))?;
        out.insert(stage_id, status);
    }
    Ok(out)
}

fn manifest_tool_statuses() -> Result<BTreeMap<String, String>> {
    let tools_dir = workspace_root()?.join("domain/fastq/tools");
    let mut out = BTreeMap::new();
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
        let tool_id = raw
            .lines()
            .find_map(|line| line.strip_prefix("tool_id: "))
            .map(|value| value.trim().trim_matches('"').to_string())
            .with_context(|| format!("tool_id missing in {}", path.display()))?;
        let status = raw
            .lines()
            .find_map(|line| line.strip_prefix("status: "))
            .map(|value| value.trim().trim_matches('"').to_string())
            .with_context(|| format!("status missing in {}", path.display()))?;
        out.insert(tool_id, status);
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

fn stage_manifest_tool_integration() -> Result<BTreeMap<String, BTreeMap<String, String>>> {
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
        let yaml: Value = bijux_dna_infra::formats::parse_yaml(
            &std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?,
        )
        .with_context(|| format!("parse {}", path.display()))?;
        let stage_id = yaml
            .get("stage_id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .with_context(|| format!("stage_id missing in {}", path.display()))?;
        let mut tool_map = BTreeMap::new();
        for tool_id in yaml
            .get("compatible_tools")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            tool_map.insert(tool_id.to_string(), "governed_contract".to_string());
        }
        for tool_id in yaml
            .get("planned_out_of_scope")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
        {
            tool_map.insert(tool_id.to_string(), "planned_contract".to_string());
        }
        out.insert(stage_id, tool_map);
    }
    Ok(out)
}

fn tool_manifest_reference_index_compatibility() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let tools_dir = workspace_root()?.join("domain/fastq/tools");
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(&tools_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let yaml: Value = bijux_dna_infra::formats::parse_yaml(
            &std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?,
        )
        .with_context(|| format!("parse {}", path.display()))?;
        let tool_id = yaml
            .get("tool_id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .with_context(|| format!("tool_id missing in {}", path.display()))?;
        let backends = yaml
            .get("reference_index_backends")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        if !backends.is_empty() {
            out.insert(tool_id, backends);
        }
    }
    Ok(out)
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

#[test]
fn generated_index_stage_tool_integration_matches_stage_manifests() -> Result<()> {
    assert_eq!(
        indexed_stage_tool_integration()?,
        stage_manifest_tool_integration()?,
        "domain/fastq/index.yaml stage_tool_integration drifted from stage manifest compatibility and planned bindings"
    );
    Ok(())
}

#[test]
fn generated_index_reference_index_compatibility_matches_tool_manifests() -> Result<()> {
    assert_eq!(
        indexed_reference_index_compatibility()?,
        tool_manifest_reference_index_compatibility()?,
        "domain/fastq/index.yaml reference_index_compatibility drifted from tool manifest compatibility"
    );
    Ok(())
}

#[test]
fn generated_index_defaults_reference_known_compatible_tools() -> Result<()> {
    let stage_tools = indexed_stage_tools()?;
    let tool_ids = indexed_tool_ids()?;
    for (stage_id, default_tool) in indexed_active_defaults()? {
        let compatible = stage_tools
            .get(&stage_id)
            .with_context(|| format!("missing stage_tool_compatibility entry for {stage_id}"))?;
        assert!(
            compatible.iter().any(|tool| tool == &default_tool),
            "active default {default_tool} must stay inside stage compatibility for {stage_id}"
        );
        assert!(
            tool_ids.contains(&default_tool),
            "active default {default_tool} for {stage_id} must stay inside tool_ids"
        );
    }
    Ok(())
}

#[test]
fn generated_index_governed_stage_ids_match_supported_stage_manifests() -> Result<()> {
    let supported = manifest_stage_statuses()?
        .into_iter()
        .filter_map(|(stage_id, status)| (status == "supported").then_some(stage_id))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        indexed_governed_stage_ids()?,
        supported,
        "domain/fastq/index.yaml governed_stage_ids drifted from supported stage manifests"
    );
    Ok(())
}

#[test]
fn generated_index_governed_tool_ids_match_supported_tool_manifests() -> Result<()> {
    let supported = manifest_tool_statuses()?
        .into_iter()
        .filter_map(|(tool_id, status)| (status == "supported").then_some(tool_id))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        indexed_governed_tool_ids()?,
        supported,
        "domain/fastq/index.yaml governed_tool_ids drifted from supported tool manifests"
    );
    Ok(())
}
