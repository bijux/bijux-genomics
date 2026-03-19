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

fn string_list(map: &Value, key: &str, stage_id: &str) -> Vec<String> {
    map.get(key)
        .and_then(Value::as_mapping)
        .and_then(|entries| entries.get(Value::String(stage_id.to_string())))
        .and_then(Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

#[test]
fn index_dependencies_match_reference_guided_fastq_stages() -> Result<()> {
    let index = parse_yaml(&workspace_root()?.join("domain/fastq/index.yaml"))?;

    let validate_prereqs = string_list(&index, "stage_prerequisites", "fastq.validate_reads");
    assert!(
        validate_prereqs.is_empty(),
        "fastq.validate_reads must not depend on reference indexing"
    );

    for stage_id in [
        "fastq.deplete_host",
        "fastq.deplete_reference_contaminants",
    ] {
        let prereqs = string_list(&index, "stage_prerequisites", stage_id);
        assert!(
            prereqs.iter().any(|stage| stage == "fastq.index_reference"),
            "{stage_id} must depend on fastq.index_reference"
        );
    }
    Ok(())
}
