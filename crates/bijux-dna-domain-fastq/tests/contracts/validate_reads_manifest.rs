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
fn validation_tools_use_generic_reads_placeholder() -> Result<()> {
    for tool_id in ["fastqvalidator", "seqtk", "fqtools"] {
        let manifest = tool_manifest(tool_id)?;
        let command_template = manifest
            .get("command_template")
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("command_template missing for {tool_id}"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        assert!(
            command_template.iter().any(|part| part == &"{{reads}}"),
            "{tool_id} must validate the generic admitted read stream placeholder"
        );
        assert!(
            !command_template.iter().any(|part| part == &"{{reads_r1}}"),
            "{tool_id} must not hard-code reads_r1 in its validation command template"
        );
    }
    Ok(())
}

#[test]
fn validation_tool_manifests_admit_optional_mate_inputs() -> Result<()> {
    for tool_id in ["fastqvalidator", "seqtk", "fqtools"] {
        let manifest = tool_manifest(tool_id)?;
        let optional_inputs = manifest
            .get("execution_contract")
            .and_then(|value| value.get("optional_inputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("optional_inputs missing for {tool_id}"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        assert_eq!(
            optional_inputs,
            vec!["reads_r2"],
            "{tool_id} must publish reads_r2 as an optional admitted validation input"
        );
        let notes = manifest
            .get("stage_contracts")
            .and_then(|value| value.get("fastq.validate_reads"))
            .and_then(|value| value.get("notes"))
            .and_then(serde_json::Value::as_str)
            .with_context(|| format!("stage notes missing for {tool_id}"))?;
        assert!(
            notes.contains("optional reads_r2 mate"),
            "{tool_id} must document optional mate handling in the governed validation contract"
        );
    }
    Ok(())
}
