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
fn governed_execution_tools_publish_complete_execution_contracts() -> Result<()> {
    for tool_id in ["fastuniq", "clumpify", "trim_galore"] {
        let manifest = tool_manifest(tool_id)?;
        assert_eq!(
            manifest.get("schema_version").and_then(serde_json::Value::as_str),
            Some("bijux.tool.v1"),
            "{tool_id} must publish the governed execution manifest schema",
        );
        assert!(
            manifest
                .get("container")
                .and_then(serde_json::Value::as_object)
                .and_then(|container| container.get("image"))
                .and_then(serde_json::Value::as_str)
                .is_some(),
            "{tool_id} must declare a container image",
        );
        assert!(
            manifest
                .get("command_template")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|template| !template.is_empty()),
            "{tool_id} must declare a non-empty command template",
        );
        assert!(
            manifest
                .get("outputs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|outputs| !outputs.is_empty()),
            "{tool_id} must declare governed outputs",
        );
        assert!(
            manifest.get("constraints").and_then(serde_json::Value::as_object).is_some(),
            "{tool_id} must declare execution constraints",
        );
        let execution_contract = manifest
            .get("execution_contract")
            .and_then(serde_json::Value::as_object)
            .with_context(|| format!("{tool_id} missing execution_contract"))?;
        assert!(
            execution_contract
                .get("expected_outputs")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|outputs| !outputs.is_empty()),
            "{tool_id} execution contract must declare expected outputs",
        );
    }
    Ok(())
}
