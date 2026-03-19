use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_domain_fastq::FASTQ_STAGE_ID_CATALOG;
use serde_yaml::Value;

#[derive(Debug, Clone)]
struct ToolManifestMeta {
    status: String,
    stage_ids: BTreeSet<String>,
    comparability_refs: BTreeSet<String>,
}

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

fn yaml_string(value: Option<&Value>) -> Option<String> {
    value.and_then(Value::as_str).map(str::to_string)
}

fn yaml_string_set(value: Option<&Value>) -> BTreeSet<String> {
    value.and_then(Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn stage_manifest_tools() -> Result<BTreeMap<String, BTreeSet<String>>> {
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
        let yaml = parse_yaml(&path)?;
        let stage_id = yaml_string(yaml.get("stage_id"))
            .with_context(|| format!("stage_id missing in {}", path.display()))?;
        let compatible_tools = yaml_string_set(yaml.get("compatible_tools"));
        out.insert(stage_id, compatible_tools);
    }
    Ok(out)
}

fn tool_manifest_meta() -> Result<BTreeMap<String, ToolManifestMeta>> {
    let tools_dir = workspace_root()?.join("domain/fastq/tools");
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(&tools_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("yaml") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let yaml = parse_yaml(&path)?;
        assert!(
            yaml.get("stage_id").is_none(),
            "{} must not use legacy stage_id",
            path.display()
        );
        assert!(
            yaml.get("role").is_none(),
            "{} must not use legacy role metadata",
            path.display()
        );
        assert!(
            yaml.get("authoritative").is_none(),
            "{} must not use legacy authoritative metadata",
            path.display()
        );
        assert!(
            yaml.get("strict_capable").is_none(),
            "{} must not use legacy strict_capable metadata",
            path.display()
        );
        let tool_id = yaml_string(yaml.get("tool_id"))
            .with_context(|| format!("tool_id missing in {}", path.display()))?;
        let status = yaml_string(yaml.get("status"))
            .with_context(|| format!("status missing in {}", path.display()))?;
        let stage_ids = yaml_string_set(yaml.get("stage_ids"));
        assert!(
            !stage_ids.is_empty(),
            "{} must declare non-empty stage_ids",
            path.display()
        );
        let comparability_refs = yaml
            .get("comparability")
            .and_then(|value| value.as_mapping())
            .map(|mapping| {
                ["comparable_with", "non_comparable_with"]
                    .into_iter()
                    .flat_map(|key| {
                        mapping
                            .get(&Value::String(key.to_string()))
                            .into_iter()
                            .flat_map(|value| yaml_string_set(Some(value)))
                    })
                    .collect::<BTreeSet<_>>()
            })
            .unwrap_or_default();
        out.insert(
            tool_id,
            ToolManifestMeta {
                status,
                stage_ids,
                comparability_refs,
            },
        );
    }
    Ok(out)
}

#[test]
fn root_stage_manifests_match_rust_fastq_catalog() -> Result<()> {
    let manifest_ids = stage_manifest_tools()?.into_keys().collect::<BTreeSet<_>>();
    let rust_ids = FASTQ_STAGE_ID_CATALOG
        .iter()
        .map(|stage| stage.to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(
        manifest_ids, rust_ids,
        "root fastq stage manifests drifted from Rust stage catalog"
    );
    Ok(())
}

#[test]
fn tool_stage_ids_reference_known_fastq_stages() -> Result<()> {
    let known_stages = FASTQ_STAGE_ID_CATALOG
        .iter()
        .map(|stage| stage.to_string())
        .collect::<BTreeSet<_>>();
    for (tool_id, meta) in tool_manifest_meta()? {
        for stage_id in meta.stage_ids {
            assert!(
                known_stages.contains(&stage_id),
                "tool {tool_id} declares unknown fastq stage {stage_id}"
            );
        }
    }
    Ok(())
}

#[test]
fn stage_compatible_tools_and_tool_stage_ids_are_symmetric() -> Result<()> {
    let stage_tools = stage_manifest_tools()?;
    let tool_meta = tool_manifest_meta()?;

    for (stage_id, tools) in &stage_tools {
        for tool_id in tools {
            let declared = tool_meta.get(tool_id).with_context(|| {
                format!("stage {stage_id} references missing tool manifest {tool_id}")
            })?;
            assert!(
                declared.stage_ids.contains(stage_id),
                "stage {stage_id} lists tool {tool_id}, but the tool manifest does not declare that stage"
            );
        }
    }

    for (tool_id, meta) in &tool_meta {
        if meta.status != "supported" {
            continue;
        }
        for stage_id in &meta.stage_ids {
            let compatible = stage_tools.get(stage_id).with_context(|| {
                format!("tool {tool_id} references missing stage manifest {stage_id}")
            })?;
            assert!(
                compatible.contains(tool_id),
                "tool {tool_id} declares stage {stage_id}, but the stage manifest does not list that tool"
            );
        }
    }

    Ok(())
}

#[test]
fn tool_comparability_refs_resolve_to_known_fastq_tools() -> Result<()> {
    let tool_meta = tool_manifest_meta()?;
    let known_tools = tool_meta.keys().cloned().collect::<BTreeSet<_>>();
    for (tool_id, meta) in tool_meta {
        for referenced_tool in meta.comparability_refs {
            assert!(
                known_tools.contains(&referenced_tool),
                "tool {tool_id} comparability metadata references missing fastq tool {referenced_tool}"
            );
        }
    }
    Ok(())
}
