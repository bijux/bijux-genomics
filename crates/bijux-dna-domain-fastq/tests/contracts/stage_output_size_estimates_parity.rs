use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn stage_manifest_required_outputs() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let stages_dir = workspace_root()?.join("domain/fastq/stages");
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(&stages_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let raw =
            std::fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        let stage_id = raw
            .lines()
            .find_map(|line| line.strip_prefix("stage_id: "))
            .map(|value| value.trim().trim_matches('"').to_string())
            .with_context(|| format!("stage_id missing in {}", path.display()))?;
        let required_outputs =
            block_list(&raw, "required_outputs").into_iter().collect::<BTreeSet<_>>();
        out.insert(stage_id, required_outputs);
    }
    Ok(out)
}

fn indexed_stage_output_size_estimates() -> Result<BTreeMap<String, BTreeSet<String>>> {
    let raw = std::fs::read_to_string(workspace_root()?.join("domain/fastq/index.yaml"))
        .context("read domain/fastq/index.yaml")?;
    let mut out = BTreeMap::<String, BTreeSet<String>>::new();
    let mut in_block = false;
    let mut current_stage = None::<String>;
    for line in raw.lines() {
        if line == "stage_output_size_estimates_mb:" {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if !line.starts_with(' ') && line.contains(':') {
            break;
        }
        if line.starts_with("  ") && !line.starts_with("    ") {
            let Some(stage) = line.strip_prefix("  ").and_then(|rest| rest.strip_suffix(':'))
            else {
                continue;
            };
            let stage = stage.to_string();
            current_stage = Some(stage.clone());
            out.entry(stage).or_default();
            continue;
        }
        if let Some(artifact) = line.strip_prefix("    ").and_then(|rest| rest.split(':').next()) {
            if let Some(stage) = &current_stage {
                out.entry(stage.clone()).or_default().insert(artifact.to_string());
            }
        }
    }
    Ok(out)
}

fn block_list(raw: &str, key: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_block = false;
    for line in raw.lines() {
        if line == format!("{key}:") {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if !line.starts_with("  - ") {
            break;
        }
        out.push(line.trim_start_matches("  - ").trim_matches('"').to_string());
    }
    out
}

#[test]
fn stage_output_size_estimates_match_required_stage_outputs() -> Result<()> {
    assert_eq!(
        indexed_stage_output_size_estimates()?,
        stage_manifest_required_outputs()?,
        "domain/fastq/index.yaml stage_output_size_estimates_mb drifted from stage manifest required_outputs"
    );
    Ok(())
}
