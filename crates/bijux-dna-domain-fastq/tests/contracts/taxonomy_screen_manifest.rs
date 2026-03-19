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
fn taxonomy_screen_manifest_keeps_read_classifier_tools_only() -> Result<()> {
    let path = workspace_root()?.join("domain/fastq/stages/screen_taxonomy.yaml");
    let yaml = parse_yaml(&path)?;
    let compatible_tools = yaml
        .get("compatible_tools")
        .and_then(Value::as_sequence)
        .context("compatible_tools")?
        .iter()
        .filter_map(Value::as_str)
        .collect::<Vec<_>>();

    assert!(
        !compatible_tools.contains(&"metaphlan"),
        "fastq.screen_taxonomy must not admit marker-profile tools under the read-classifier contract"
    );
    assert!(
        !compatible_tools.contains(&"fastq_screen"),
        "fastq.screen_taxonomy must not admit mapping-QC tools under the read-classifier contract"
    );
    Ok(())
}
