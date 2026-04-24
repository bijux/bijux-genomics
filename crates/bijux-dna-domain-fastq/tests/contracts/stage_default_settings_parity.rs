use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn declared_only_stage_ids() -> BTreeSet<String> {
    bijux_dna_domain_fastq::execution_declared_only_stage_ids()
        .into_iter()
        .map(|stage_id| stage_id.as_str().to_string())
        .collect()
}

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

fn indexed_stage_tool_compatibility() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let raw = std::fs::read_to_string(workspace_root()?.join("domain/fastq/index.yaml"))
        .context("read domain/fastq/index.yaml")?;
    let mut out = BTreeMap::<String, BTreeSet<String>>::new();
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
        if line.starts_with("  ") && !line.starts_with("  - ") {
            let Some((stage, inline_value)) =
                line.strip_prefix("  ").and_then(|rest| rest.split_once(':'))
            else {
                continue;
            };
            let stage = stage.to_string();
            current_stage = (inline_value.trim() != "[]").then_some(stage.clone());
            out.entry(stage).or_default();
            continue;
        }
        if let Some(tool) = line.strip_prefix("  - ") {
            if let Some(stage) = &current_stage {
                out.entry(stage.clone()).or_default().insert(tool.to_string());
            }
        }
    }
    Ok(out)
}

fn indexed_stage_default_settings() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let raw = std::fs::read_to_string(workspace_root()?.join("domain/fastq/index.yaml"))
        .context("read domain/fastq/index.yaml")?;
    let mut out = BTreeMap::<String, BTreeSet<String>>::new();
    let mut in_block = false;
    let mut current_stage = None::<String>;
    for line in raw.lines() {
        if line == "stage_default_settings:" {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if !line.starts_with(' ') && line.contains(':') {
            break;
        }
        if line.starts_with("    ") {
            if let Some(tool) = line.strip_prefix("    ").and_then(|rest| rest.strip_suffix(':')) {
                if let Some(stage) = &current_stage {
                    out.entry(stage.clone()).or_default().insert(tool.to_string());
                }
            }
            continue;
        }
        if let Some(stage) = line.strip_prefix("  ").and_then(|rest| rest.strip_suffix(':')) {
            if stage.starts_with("fastq.") {
                let stage = stage.to_string();
                current_stage = Some(stage.clone());
                out.entry(stage).or_default();
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
fn stage_default_settings_cover_current_supported_tools() -> Result<()> {
    assert_eq!(
        indexed_stage_tool_compatibility()?,
        stage_manifest_tools()?,
        "domain/fastq/index.yaml stage_tool_compatibility drifted from stage manifest compatible_tools"
    );
    Ok(())
}

#[test]
fn stage_default_settings_only_reference_current_compatible_tools() -> Result<()> {
    let stage_tools = indexed_stage_tool_compatibility()?;
    let declared_only = declared_only_stage_ids();
    for (stage_id, configured_tools) in indexed_stage_default_settings()? {
        if declared_only.contains(&stage_id) {
            continue;
        }
        let compatible = stage_tools
            .get(&stage_id)
            .with_context(|| format!("missing stage_tool_compatibility entry for {stage_id}"))?;
        assert_eq!(
            compatible, &configured_tools,
            "domain/fastq/index.yaml stage_default_settings drifted from current compatible_tools for {stage_id}"
        );
    }
    Ok(())
}

#[test]
fn declared_only_stages_do_not_receive_runtime_default_settings() -> Result<()> {
    let declared_only = declared_only_stage_ids();
    for stage_id in indexed_stage_default_settings()?.keys() {
        assert!(
            !declared_only.contains(stage_id),
            "declared-only stage {stage_id} must not receive generated runtime default settings"
        );
    }
    Ok(())
}
