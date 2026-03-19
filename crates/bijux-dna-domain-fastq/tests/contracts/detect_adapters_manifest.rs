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
    let raw = std::fs::read_to_string(path)
        .with_context(|| format!("read {}", path.display()))?;
    serde_yaml::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

#[test]
fn detect_adapters_manifest_accepts_optional_read_two() -> Result<()> {
    let path = workspace_root()?.join("domain/fastq/stages/detect_adapters.yaml");
    let yaml = parse_yaml(&path)?;
    let inputs = yaml
        .get("inputs")
        .and_then(Value::as_sequence)
        .context("inputs")?;
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
        .and_then(Value::as_sequence)
        .context("allowed_missingness")?
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();
    assert!(
        allowed_missingness.contains(&"reads_r2"),
        "detect_adapters manifest must allow missing reads_r2 for single-end input"
    );
    Ok(())
}
