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

fn stage_manifest(stage_name: &str) -> Result<serde_json::Value> {
    let path = workspace_root()?.join(format!("domain/fastq/stages/{stage_name}.yaml"));
    let raw = std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    bijux_dna_infra::formats::parse_yaml(&raw).with_context(|| format!("parse {}", path.display()))
}

#[test]
fn validation_tools_use_generic_reads_placeholder() -> Result<()> {
    for tool_id in ["fastqvalidator", "seqtk", "fqtools", "fastq_scan"] {
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
fn fastqc_validation_contract_publishes_governed_artifacts() -> Result<()> {
    let manifest = tool_manifest("fastqc")?;
    let stage_ids = manifest
        .get("stage_ids")
        .and_then(serde_json::Value::as_array)
        .context("stage_ids missing for fastqc")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        stage_ids.contains(&"fastq.validate_reads"),
        "fastqc must publish fastq.validate_reads in its governed stage list"
    );

    let validate_contract = manifest
        .get("stage_contracts")
        .and_then(|value| value.get("fastq.validate_reads"))
        .context("fastqc validate stage contract missing")?;
    let expected_artifacts = validate_contract
        .get("expected_artifacts")
        .and_then(serde_json::Value::as_array)
        .context("fastqc validate expected_artifacts missing")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        expected_artifacts,
        vec!["validation_report", "validated_reads_manifest"],
        "fastqc validate contract must publish the governed validation artifacts"
    );

    let command_template = manifest
        .get("command_template")
        .and_then(serde_json::Value::as_array)
        .context("command_template missing for fastqc")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        command_template.iter().any(|part| part == &"{{reads_r1}}"),
        "fastqc keeps its native reads_r1-oriented command surface and relies on the governed adapter for validate rendering"
    );
    Ok(())
}

#[test]
fn validation_tool_manifests_admit_optional_mate_inputs() -> Result<()> {
    for tool_id in ["fastqvalidator", "fastqc", "fastq_scan", "seqtk", "fqtools"] {
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
        let stage_optional_inputs = manifest
            .get("stage_contracts")
            .and_then(|value| value.get("fastq.validate_reads"))
            .and_then(|value| value.get("optional_inputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("stage optional_inputs missing for {tool_id}"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        assert_eq!(
            stage_optional_inputs,
            vec!["reads_r2"],
            "{tool_id} must publish reads_r2 as an optional stage-family validation input"
        );
        assert!(
            notes.contains("optional reads_r2 mate"),
            "{tool_id} must document optional mate handling in the governed validation contract"
        );
    }
    Ok(())
}

#[test]
fn validate_stage_manifest_lists_all_supported_backends() -> Result<()> {
    let manifest = stage_manifest("validate_reads")?;
    let compatible_tools = manifest
        .get("compatible_tools")
        .and_then(serde_json::Value::as_array)
        .context("compatible_tools missing")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(
        compatible_tools,
        vec!["fastqvalidator", "fastqc", "fastq_scan", "seqtk", "fqtools"],
        "validate stage manifest must publish the full governed backend surface"
    );
    Ok(())
}

#[test]
fn validate_stage_manifest_uses_single_stream_ports() -> Result<()> {
    let manifest = stage_manifest("validate_reads")?;
    let inputs =
        manifest.get("inputs").and_then(serde_json::Value::as_array).context("inputs missing")?;
    for input_name in ["reads_r1", "reads_r2"] {
        let input = inputs
            .iter()
            .find(|entry| entry.get("name").and_then(serde_json::Value::as_str) == Some(input_name))
            .with_context(|| format!("missing input {input_name}"))?;
        assert_eq!(
            input.get("cardinality").and_then(serde_json::Value::as_str),
            Some("One"),
            "{input_name} must be modeled as a single stream port"
        );
    }
    Ok(())
}

#[test]
fn validate_stage_manifest_documents_layout_derived_pair_sync_default() -> Result<()> {
    let manifest = stage_manifest("validate_reads")?;
    let params = manifest
        .get("parameters")
        .and_then(serde_json::Value::as_array)
        .context("parameters missing")?;
    let pair_sync_policy = params
        .iter()
        .find(|entry| {
            entry.get("name").and_then(serde_json::Value::as_str) == Some("pair_sync_policy")
        })
        .context("pair_sync_policy parameter missing")?;
    assert_eq!(
        pair_sync_policy.get("default").and_then(serde_json::Value::as_str),
        Some("layout_derived"),
        "validate stage contract must document layout-derived pair sync defaults"
    );
    let assumptions = manifest
        .get("assumptions")
        .and_then(serde_json::Value::as_array)
        .context("assumptions missing")?;
    assert!(
        assumptions.iter().filter_map(serde_json::Value::as_str).any(|entry| {
            entry.contains("pair_sync_policy defaults to require_header_sync")
                && entry.contains("not_applicable")
        }),
        "validate stage assumptions must describe paired and single-end default resolution"
    );
    Ok(())
}
