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
fn admitted_deduplicate_tools_only_compare_with_stage_peers() -> Result<()> {
    let remove_duplicates_tools = ["fastuniq", "clumpify"];
    for tool_id in remove_duplicates_tools {
        let manifest = tool_manifest(tool_id)?;
        let comparable_with = manifest
            .get("comparability")
            .and_then(|value| value.get("comparable_with"))
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("comparability list missing for {tool_id}"))?;
        for comparable_tool in comparable_with {
            let comparable_tool = comparable_tool
                .as_str()
                .with_context(|| format!("non-string comparable_with entry for {tool_id}"))?;
            assert!(
                remove_duplicates_tools.contains(&comparable_tool),
                "deduplicate tool {tool_id} must not reference non-admitted remove_duplicates peer {comparable_tool}",
            );
        }
    }
    Ok(())
}

#[test]
fn fastuniq_manifest_requires_paired_remove_duplicates_inputs() -> Result<()> {
    let manifest = tool_manifest("fastuniq")?;
    let required_inputs = manifest
        .get("execution_contract")
        .and_then(|value| value.get("required_inputs"))
        .and_then(serde_json::Value::as_array)
        .context("fastuniq execution required_inputs")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(required_inputs, vec!["reads_r1", "reads_r2"]);

    let stage_required_inputs = manifest
        .get("stage_contracts")
        .and_then(|value| value.get("fastq.remove_duplicates"))
        .and_then(|value| value.get("required_inputs"))
        .and_then(serde_json::Value::as_array)
        .context("fastuniq stage required_inputs")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(stage_required_inputs, vec!["reads_r1", "reads_r2"]);
    Ok(())
}

#[test]
fn clumpify_manifest_keeps_single_end_remove_duplicates_contract() -> Result<()> {
    let manifest = tool_manifest("clumpify")?;
    let expected_outputs = manifest
        .get("execution_contract")
        .and_then(|value| value.get("expected_outputs"))
        .and_then(serde_json::Value::as_array)
        .context("clumpify execution expected_outputs")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(expected_outputs, vec!["dedup_reads_r1", "report_json"]);

    let stage_expected_artifacts = manifest
        .get("stage_contracts")
        .and_then(|value| value.get("fastq.remove_duplicates"))
        .and_then(|value| value.get("expected_artifacts"))
        .and_then(serde_json::Value::as_array)
        .context("clumpify stage expected_artifacts")?
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect::<Vec<_>>();
    assert_eq!(stage_expected_artifacts, vec!["dedup_reads_r1", "report_json"]);
    Ok(())
}
