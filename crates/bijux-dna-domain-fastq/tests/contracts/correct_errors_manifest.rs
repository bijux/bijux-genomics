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
fn correction_tool_manifests_publish_optional_mate_inputs() -> Result<()> {
    for tool_id in ["rcorrector", "musket", "lighter", "bayeshammer"] {
        let manifest = tool_manifest(tool_id)?;
        let required_inputs = manifest
            .get("execution_contract")
            .and_then(|value| value.get("required_inputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} execution required_inputs"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        let optional_inputs = manifest
            .get("execution_contract")
            .and_then(|value| value.get("optional_inputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} execution optional_inputs"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        assert_eq!(
            required_inputs,
            vec!["reads_r1"],
            "{tool_id} execution contract must require only reads_r1 for the governed correction stage"
        );
        assert_eq!(
            optional_inputs,
            vec!["reads_r2"],
            "{tool_id} execution contract must admit reads_r2 as an optional mate input"
        );
    }
    Ok(())
}

#[test]
fn correction_tool_capabilities_match_current_stage_runtime_surface() -> Result<()> {
    for tool_id in ["rcorrector", "musket", "lighter", "bayeshammer"] {
        let manifest = tool_manifest(tool_id)?;
        let capabilities = manifest
            .get("capabilities")
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} capabilities"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        assert_eq!(
            capabilities,
            vec!["SE", "PE"],
            "{tool_id} capability declaration must match the governed single-end and paired-end correct_errors stage contract"
        );
    }
    Ok(())
}
