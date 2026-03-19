use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

fn workspace_root() -> Result<PathBuf> {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("workspace root")
}

fn stage_manifest_tools() -> Result<BTreeMap<String, Vec<String>>> {
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
        let raw = std::fs::read_to_string(&path)
            .with_context(|| format!("read {}", path.display()))?;
        let stage_id = raw
            .lines()
            .find_map(|line| line.strip_prefix("stage_id: "))
            .map(|value| value.trim().trim_matches('"').to_string())
            .with_context(|| format!("stage_id missing in {}", path.display()))?;
        let compatible_tools = block_list(&raw, "compatible_tools");
        out.insert(stage_id, compatible_tools);
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
        out.push(line.trim_start_matches("  - ").to_string());
    }
    out
}

#[test]
fn stage_contract_tool_ids_match_stage_manifests() -> Result<()> {
    for (stage_id, expected_tools) in stage_manifest_tools()? {
        let Some(json) = bijux_dna_domain_fastq::stage_contract_json(&stage_id) else {
            continue;
        };
        let actual_tools = json
            .get("tool_ids")
            .and_then(serde_json::Value::as_array)
            .with_context(|| format!("tool_ids missing for {stage_id}"))?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(ToOwned::to_owned)
                    .with_context(|| format!("non-string tool_id in {stage_id}"))
            })
            .collect::<Result<Vec<_>>>()?;
        assert_eq!(
            actual_tools, expected_tools,
            "stage contract tool_ids drifted from stage manifest compatible_tools for {stage_id}"
        );
    }
    Ok(())
}
