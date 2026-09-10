use crate::domain::compilation::{FastqToolContractEntry, ToolRegistryEntry};
use crate::io::{list_yaml_files, read_utf8};
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use toml::Value as TomlValue;

#[derive(Deserialize)]
struct StageContractDoc {
    stage_id: Option<String>,
    #[serde(default)]
    planned_out_of_scope: Vec<String>,
}

pub(super) fn load_planned_out_of_scope(
    stage_dir: &Path,
) -> Result<BTreeMap<String, BTreeSet<String>>> {
    let mut map = BTreeMap::<String, BTreeSet<String>>::new();
    for path in list_yaml_files(stage_dir)? {
        let raw = read_utf8(&path)?;
        let stage: StageContractDoc = bijux_dna_infra::formats::yaml::parse_yaml(&raw)
            .map_err(|err| anyhow!("parse {}: {err}", path.display()))?;
        let Some(stage_id) = stage.stage_id else {
            continue;
        };
        if !stage.planned_out_of_scope.is_empty() {
            map.insert(stage_id, stage.planned_out_of_scope.into_iter().collect());
        }
    }
    Ok(map)
}
pub(super) fn load_tool_registry(path: &Path) -> Result<BTreeMap<String, ToolRegistryEntry>> {
    let raw = read_utf8(path)?;
    let root: TomlValue =
        toml::from_str(&raw).with_context(|| format!("parse TOML {}", path.display()))?;
    let mut out = BTreeMap::new();
    for row in root.get("tools").and_then(TomlValue::as_array).cloned().unwrap_or_default() {
        let Some(table) = row.as_table() else {
            continue;
        };
        let Some(tool_id) = table
            .get("tool_id")
            .or_else(|| table.get("id"))
            .and_then(TomlValue::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        else {
            continue;
        };
        let runtimes = table
            .get("runtimes")
            .and_then(TomlValue::as_array)
            .into_iter()
            .flatten()
            .filter_map(TomlValue::as_str)
            .map(str::to_string)
            .collect::<Vec<_>>();
        out.insert(
            tool_id.to_string(),
            ToolRegistryEntry {
                status: table
                    .get("status")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                version: table
                    .get("version")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                default_version: table
                    .get("default_version")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                version_rule: table
                    .get("version_rule")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                upstream: table
                    .get("upstream")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                citation: table
                    .get("citation")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                license: table
                    .get("license")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                pinned_commit: table
                    .get("pinned_commit")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                pin_strategy: table
                    .get("pin_strategy")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                runtimes,
                container_ref: table
                    .get("container_ref")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                dockerfile: table
                    .get("dockerfile")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
                apptainer_def: table
                    .get("apptainer_def")
                    .and_then(TomlValue::as_str)
                    .unwrap_or_default()
                    .to_string(),
            },
        );
    }
    Ok(out)
}
pub(super) fn load_fastq_tool_contracts(
    path: &Path,
) -> Result<BTreeMap<String, FastqToolContractEntry>> {
    let mut out = BTreeMap::new();
    for tool_path in list_yaml_files(path)? {
        let raw = read_utf8(&tool_path)?;
        let entry: FastqToolContractEntry = bijux_dna_infra::formats::yaml::parse_yaml(&raw)
            .map_err(|err| anyhow!("parse {}: {err}", tool_path.display()))?;
        if !entry.tool_id.trim().is_empty() {
            out.insert(entry.tool_id.clone(), entry);
        }
    }
    Ok(out)
}
