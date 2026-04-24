use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::Value;

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn parse_yaml(path: &Path) -> Result<Value> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    bijux_dna_infra::formats::parse_yaml(&raw).with_context(|| format!("parse {}", path.display()))
}

#[test]
fn detect_adapters_manifest_accepts_optional_read_two() -> Result<()> {
    let path = workspace_root()?.join("domain/fastq/stages/detect_adapters.yaml");
    let yaml = parse_yaml(&path)?;
    let inputs = yaml.get("inputs").and_then(Value::as_array).context("inputs")?;
    let input_names = inputs
        .iter()
        .filter_map(|input| input.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(
        input_names.contains(&"reads_r2"),
        "detect_adapters manifest must declare reads_r2 for paired input"
    );

    let allowed_missingness = yaml
        .get("allowed_missingness")
        .and_then(Value::as_array)
        .context("allowed_missingness")?
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        allowed_missingness.contains(&"reads_r2"),
        "detect_adapters manifest must allow missing reads_r2 for single-end input"
    );
    let output_names = yaml
        .get("outputs")
        .and_then(Value::as_array)
        .context("outputs")?
        .iter()
        .filter_map(|output| output.get("name").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(
        output_names.contains(&"adapter_evidence_dir"),
        "detect_adapters manifest must expose the governed adapter evidence directory"
    );
    assert!(
        output_names.contains(&"report_json"),
        "detect_adapters manifest must expose the canonical governed report_json output"
    );
    assert!(
        allowed_missingness.contains(&"adapter_evidence_dir"),
        "detect_adapters manifest must allow missing adapter_evidence_dir when only the normalized report is materialized"
    );
    Ok(())
}
