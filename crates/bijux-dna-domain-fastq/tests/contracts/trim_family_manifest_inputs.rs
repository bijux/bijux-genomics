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
fn trim_polyg_tool_manifests_publish_optional_mate_inputs() -> Result<()> {
    for tool_id in ["fastp", "bbduk"] {
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
        let notes = manifest
            .get("stage_contracts")
            .and_then(|value| value.get("fastq.trim_polyg_tails"))
            .and_then(|value| value.get("notes"))
            .and_then(serde_json::Value::as_str)
            .with_context(|| format!("{tool_id} fastq.trim_polyg_tails notes"))?;

        assert_eq!(
            required_inputs,
            vec!["reads_r1"],
            "{tool_id} must keep reads_r1 as the canonical required trim_polyg input"
        );
        assert_eq!(
            optional_inputs,
            vec!["reads_r2"],
            "{tool_id} must publish reads_r2 as an optional mate input for trim_polyg_tails"
        );
        assert!(
            notes.contains("optional reads_r2 mate"),
            "{tool_id} trim_polyg_tails notes must document optional mate handling"
        );
    }
    Ok(())
}

#[test]
fn terminal_damage_tool_manifests_publish_optional_mate_inputs() -> Result<()> {
    for tool_id in ["cutadapt", "seqkit"] {
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
        let notes = manifest
            .get("stage_contracts")
            .and_then(|value| value.get("fastq.trim_terminal_damage"))
            .and_then(|value| value.get("notes"))
            .and_then(serde_json::Value::as_str)
            .with_context(|| format!("{tool_id} fastq.trim_terminal_damage notes"))?;

        assert_eq!(
            required_inputs,
            vec!["reads_r1"],
            "{tool_id} must keep reads_r1 as the canonical required terminal-damage input"
        );
        assert_eq!(
            optional_inputs,
            vec!["reads_r2"],
            "{tool_id} must publish reads_r2 as an optional mate input for trim_terminal_damage"
        );
        assert!(
            notes.contains("optional reads_r2 mate"),
            "{tool_id} trim_terminal_damage notes must document optional mate handling"
        );
    }
    Ok(())
}

#[test]
fn seqkit_manifest_declares_paired_trim_runtime_capability() -> Result<()> {
    let manifest = tool_manifest("seqkit")?;
    let capabilities = manifest
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .context("seqkit capabilities")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();

    assert!(
        capabilities.contains(&"SE"),
        "seqkit must keep single-end capability for its governed FASTQ stages"
    );
    assert!(
        capabilities.contains(&"PE"),
        "seqkit must advertise paired-end capability for trim and terminal-damage families"
    );
    Ok(())
}

#[test]
fn seqpurge_manifest_declares_paired_trim_runtime_contract() -> Result<()> {
    let manifest = tool_manifest("seqpurge")?;
    let capabilities = manifest
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .context("seqpurge capabilities")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    let required_inputs = manifest
        .get("execution_contract")
        .and_then(|value| value.get("required_inputs"))
        .and_then(serde_json::Value::as_array)
        .context("seqpurge execution required_inputs")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();

    assert_eq!(capabilities, vec!["PE"]);
    assert_eq!(required_inputs, vec!["reads_r1", "reads_r2"]);
    Ok(())
}
