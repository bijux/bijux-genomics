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
fn governed_trim_wrappers_publish_optional_mate_outputs() -> Result<()> {
    for tool_id in ["fastp", "trim_galore"] {
        let manifest = tool_manifest(tool_id)?;
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
            expected_outputs,
            vec!["trimmed_reads_r1", "report_json"],
            "{tool_id} must require only the always-emitted governed trim outputs"
        );
        assert_eq!(
            optional_outputs,
            vec!["trimmed_reads_r2"],
            "{tool_id} must publish trimmed_reads_r2 as an optional mate output"
        );
    }
    Ok(())
}
