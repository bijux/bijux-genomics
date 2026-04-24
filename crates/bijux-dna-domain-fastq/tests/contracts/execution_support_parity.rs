use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

type ExecutionSupportManifestRow = (String, String, Option<String>, BTreeSet<String>);

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn stage_manifest_tools() -> Result<BTreeMap<String, BTreeSet<String>>> {
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
        let compatible_tools =
            block_list(&raw, "compatible_tools").into_iter().collect::<BTreeSet<_>>();
        out.insert(stage_id, compatible_tools);
    }
    Ok(out)
}

fn stage_manifest_planned_tools() -> Result<BTreeMap<String, BTreeSet<String>>> {
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
        let planned_tools =
            block_list(&raw, "planned_out_of_scope").into_iter().collect::<BTreeSet<_>>();
        out.insert(stage_id, planned_tools);
    }
    Ok(out)
}

fn execution_support_manifest() -> Result<Vec<ExecutionSupportManifestRow>> {
    let raw =
        std::fs::read_to_string(workspace_root()?.join("domain/fastq/execution_support.yaml"))
            .context("read domain/fastq/execution_support.yaml")?;
    let yaml: Value = bijux_dna_infra::formats::parse_yaml(&raw)
        .context("parse domain/fastq/execution_support.yaml")?;
    let stages =
        yaml.get("stages").and_then(Value::as_array).context("execution_support stages")?;
    let mut out = Vec::new();
    for stage in stages {
        let stage_id = stage
            .get("stage_id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .context("execution_support stage_id")?;
        let default_tool = stage.get("default_tool").and_then(Value::as_str).map(str::to_string);
        let execution_status = stage
            .get("execution_status")
            .and_then(Value::as_str)
            .map(str::to_string)
            .context("execution_support execution_status")?;
        let admitted_tools = stage
            .get("admitted_tools")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect::<BTreeSet<_>>();
        out.push((stage_id, execution_status, default_tool, admitted_tools));
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
        out.push(line.trim_start_matches("  - ").trim_matches('"').to_string());
    }
    out
}

#[test]
fn execution_support_stays_inside_stage_tool_contracts() -> Result<()> {
    let stage_tools = stage_manifest_tools()?;
    for (stage_id, _execution_status, default_tool, admitted_tools) in execution_support_manifest()?
    {
        let compatible = stage_tools
            .get(&stage_id)
            .with_context(|| format!("missing stage manifest compatible_tools for {stage_id}"))?;
        for tool in &admitted_tools {
            assert!(
                compatible.contains(tool),
                "execution support admitted tool {tool} must remain inside compatible_tools for {stage_id}"
            );
        }
        if let Some(default_tool) = default_tool {
            assert!(
                compatible.contains(&default_tool),
                "execution support default tool {default_tool} must remain inside compatible_tools for {stage_id}"
            );
            assert!(
                admitted_tools.contains(&default_tool),
                "execution support default tool {default_tool} must remain inside admitted_tools for {stage_id}"
            );
        }
    }
    Ok(())
}

#[test]
fn closed_stage_contracts_match_execution_support_surface() -> Result<()> {
    let stage_tools = stage_manifest_tools()?;
    for (stage_id, execution_status, _default_tool, admitted_tools) in execution_support_manifest()?
    {
        if execution_status != "closed" {
            continue;
        }
        let compatible = stage_tools
            .get(&stage_id)
            .with_context(|| format!("missing stage manifest compatible_tools for {stage_id}"))?;
        assert_eq!(
            compatible, &admitted_tools,
            "closed runtime stage {stage_id} must keep compatible_tools aligned with execution_support admitted_tools"
        );
    }
    Ok(())
}

#[test]
fn declared_only_stage_manifests_keep_runtime_tools_out_of_compatible_tools() -> Result<()> {
    let stage_tools = stage_manifest_tools()?;
    let planned_tools = stage_manifest_planned_tools()?;
    for (stage_id, execution_status, _default_tool, admitted_tools) in execution_support_manifest()?
    {
        if execution_status != "declared_only" {
            continue;
        }
        let compatible = stage_tools
            .get(&stage_id)
            .with_context(|| format!("missing stage manifest compatible_tools for {stage_id}"))?;
        assert!(
            compatible.is_empty(),
            "declared-only stage {stage_id} must not expose governed compatible_tools",
        );
        assert!(
            admitted_tools.is_empty(),
            "declared-only stage {stage_id} must not admit runtime tools",
        );
        let planned = planned_tools.get(&stage_id).with_context(|| {
            format!("missing stage manifest planned_out_of_scope for {stage_id}")
        })?;
        assert!(
            !planned.is_empty(),
            "declared-only stage {stage_id} should keep planned tool intent in planned_out_of_scope",
        );
    }
    Ok(())
}
