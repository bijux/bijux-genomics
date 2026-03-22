use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_yaml::Value;

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn parse_yaml(path: &Path) -> Result<Value> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

#[test]
fn rrna_depletion_tool_manifest_admits_optional_mate_inputs() -> Result<()> {
    let path = workspace_root()?.join("domain/fastq/tools/sortmerna.yaml");
    let yaml = parse_yaml(&path)?;
    let optional_inputs = yaml
        .get("stage_contracts")
        .and_then(|value| value.get("fastq.deplete_rrna"))
        .and_then(|value| value.get("optional_inputs"))
        .and_then(Value::as_sequence)
        .context("stage_contracts.fastq.deplete_rrna.optional_inputs")?
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        optional_inputs.contains(&"reads_r2"),
        "sortmerna must admit reads_r2 as an optional governed rrna-depletion input"
    );
    Ok(())
}

#[test]
fn rrna_depletion_tool_manifest_documents_governed_report_contract() -> Result<()> {
    let path = workspace_root()?.join("domain/fastq/tools/sortmerna.yaml");
    let yaml = parse_yaml(&path)?;
    let notes = yaml
        .get("stage_contracts")
        .and_then(|value| value.get("fastq.deplete_rrna"))
        .and_then(|value| value.get("notes"))
        .and_then(Value::as_str)
        .context("stage_contracts.fastq.deplete_rrna.notes")?;
    assert!(
        notes.contains("normalized depletion reports"),
        "sortmerna rrna stage contract must document the governed report surface"
    );
    Ok(())
}
