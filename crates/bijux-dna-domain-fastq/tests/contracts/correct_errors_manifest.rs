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
        let expected_outputs = manifest
            .get("execution_contract")
            .and_then(|value| value.get("expected_outputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} execution expected_outputs"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        let optional_outputs = manifest
            .get("execution_contract")
            .and_then(|value| value.get("optional_outputs"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} execution optional_outputs"))?
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
        assert_eq!(
            expected_outputs,
            vec!["corrected_reads_r1", "report_json"],
            "{tool_id} execution contract must require only the always-emitted correction outputs"
        );
        assert_eq!(
            optional_outputs,
            vec!["corrected_reads_r2"],
            "{tool_id} execution contract must publish corrected_reads_r2 as an optional mate output"
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

#[test]
fn correction_tool_command_templates_follow_tool_native_workdirs() -> Result<()> {
    let rcorrector = tool_manifest("rcorrector")?;
    let rcorrector_template = rcorrector
        .get("command_template")
        .and_then(serde_json::Value::as_array)
        .context("rcorrector command_template")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert!(rcorrector_template.iter().any(|part| part == &"run_rcorrector.pl"));
    assert!(rcorrector_template.iter().any(|part| part == &"{{corrected_reads_dir}}"));

    let musket = tool_manifest("musket")?;
    let musket_template = musket
        .get("command_template")
        .and_then(serde_json::Value::as_array)
        .context("musket command_template")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert!(musket_template.iter().any(|part| part == &"-omulti"));
    assert!(musket_template.iter().any(|part| part.contains("{{corrected_reads_dir}}")));

    for tool_id in ["lighter", "bayeshammer"] {
        let manifest = tool_manifest(tool_id)?;
        let template = manifest
            .get("command_template")
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("{tool_id} command_template"))?
            .iter()
            .filter_map(serde_json::Value::as_str)
            .collect::<Vec<_>>();
        assert!(
            template.iter().any(|part| part == &"{{corrected_reads_dir}}"),
            "{tool_id} must target a governed correction work directory"
        );
        assert!(
            !template
                .iter()
                .any(|part| part == &"{{corrected_reads_r1}}"),
            "{tool_id} must not pretend to emit corrected FASTQ paths directly from the tool command"
        );
    }
    Ok(())
}
