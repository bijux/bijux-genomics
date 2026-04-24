use std::collections::{BTreeMap, BTreeSet};
use std::fs;

use anyhow::{anyhow, Context, Result};
use toml::Value as TomlValue;

use super::domain_workflow::{
    domain_directories, failure_block, inline_list, read_utf8, scalar_from_text, success_line,
    yaml_files,
};
use super::load_toml;
use super::schema_policy::external_tools;
use crate::model::domain::DomainCommandOutcome;
use crate::runtime::workspace::Workspace;

pub(super) fn check_shared_tools(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let config = load_toml(&workspace.path("configs/domain/shared_tools.toml"))?;
    let shared =
        config.get("shared_tools").and_then(TomlValue::as_table).cloned().unwrap_or_default();
    let mut tools = BTreeMap::<String, Vec<BTreeMap<String, String>>>::new();
    for dom_dir in domain_directories(workspace)? {
        let domain = dom_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid domain directory {}", dom_dir.display()))?
            .to_string();
        for tool_file in yaml_files(&dom_dir.join("tools"))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&tool_file)?;
            let Some(tool_id) = scalar_from_text(&text, "tool_id")? else {
                continue;
            };
            let mut row = BTreeMap::new();
            row.insert("domain".to_string(), domain.clone());
            row.insert(
                "default_version".to_string(),
                scalar_from_text(&text, "default_version")?.unwrap_or_default(),
            );
            row.insert(
                "license".to_string(),
                scalar_from_text(&text, "license")?.unwrap_or_default(),
            );
            row.insert(
                "upstream".to_string(),
                scalar_from_text(&text, "upstream")?.unwrap_or_default(),
            );
            row.insert("path".to_string(), workspace.rel(&tool_file).display().to_string());
            tools.entry(tool_id).or_default().push(row);
        }
    }

    let mut errors = Vec::new();
    for (tool_id, rows) in tools {
        if rows.len() <= 1 {
            continue;
        }
        let Some(shared_entry) = shared.get(&tool_id).and_then(TomlValue::as_table) else {
            errors.push(format!(
                "{tool_id}: appears in multiple domains but not declared in configs/domain/shared_tools.toml"
            ));
            continue;
        };
        let domains_declared = shared_entry
            .get("domains")
            .and_then(TomlValue::as_array)
            .map(|array| {
                array
                    .iter()
                    .filter_map(TomlValue::as_str)
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let mut domains_actual =
            rows.iter().filter_map(|row| row.get("domain").cloned()).collect::<Vec<_>>();
        let mut declared_sorted = domains_declared;
        declared_sorted.sort();
        domains_actual.sort();
        if domains_actual != declared_sorted {
            errors.push(format!(
                "{tool_id}: shared domains mismatch declared={declared_sorted:?} actual={domains_actual:?}"
            ));
        }
        for key in ["default_version", "license", "upstream"] {
            let expected = shared_entry
                .get(key)
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if expected.is_empty() {
                errors.push(format!("{tool_id}: missing {key} in shared_tools config"));
                continue;
            }
            for row in &rows {
                if let (Some(path), Some(actual)) = (row.get("path"), row.get(key)) {
                    if !actual.is_empty() && actual != &expected {
                        errors.push(format!(
                            "{path}: {key}={actual} differs from shared_tools.{tool_id}.{key}={expected}"
                        ));
                    }
                }
            }
        }
    }
    if errors.is_empty() {
        return success_line("shared-tools: OK");
    }
    failure_block("shared-tools check failed", errors)
}

pub(super) fn check_tool_container_parity(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let external = external_tools(workspace)?;
    let docker_tools = fs::read_dir(workspace.path("containers/docker/arm64"))
        .with_context(|| format!("read {}", workspace.path("containers/docker/arm64").display()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.strip_prefix("Dockerfile.").map(ToString::to_string)
        })
        .collect::<BTreeSet<_>>();
    let apptainer_tools = fs::read_dir(workspace.path("containers/apptainer/shared"))
        .with_context(|| {
            format!("read {}", workspace.path("containers/apptainer/shared").display())
        })?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            if entry.path().extension().and_then(|ext| ext.to_str()) == Some("def") {
                entry.path().file_stem().and_then(|name| name.to_str()).map(ToString::to_string)
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();
    let all_container_tools =
        docker_tools.into_iter().chain(apptainer_tools).collect::<BTreeSet<_>>();

    let mut errors = Vec::new();
    let mut declared_tools = BTreeSet::new();
    for dom_dir in domain_directories(workspace)? {
        for tool_file in yaml_files(&dom_dir.join("tools"))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&tool_file)?;
            let tool_id = scalar_from_text(&text, "tool_id")?;
            let status = scalar_from_text(&text, "status")?.unwrap_or_default();
            if let Some(tool_id_value) = tool_id.clone() {
                declared_tools.insert(tool_id_value.clone());
                if status != "out_of_scope" && !external.contains(&tool_id_value) {
                    let candidates = [tool_id_value.clone(), tool_id_value.replace('-', "_")];
                    if candidates.iter().all(|candidate| !all_container_tools.contains(candidate)) {
                        errors.push(format!(
                            "{}: tool_id '{}' has no matching container def (add container or mark in configs/domain/external_tools.toml)",
                            workspace.rel(&tool_file).display(),
                            tool_id_value
                        ));
                    }
                }
            }
        }
        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&stage_file)?;
            for tool_id in inline_list(&text, "compatible_tools")? {
                if !declared_tools.contains(&tool_id) && !external.contains(&tool_id) {
                    errors.push(format!(
                        "{}: compatible_tools references undeclared tool '{}'",
                        workspace.rel(&stage_file).display(),
                        tool_id
                    ));
                    continue;
                }
                if external.contains(&tool_id) {
                    continue;
                }
                let candidates = [tool_id.clone(), tool_id.replace('-', "_")];
                if candidates.iter().all(|candidate| !all_container_tools.contains(candidate)) {
                    errors.push(format!(
                        "{}: compatible_tools tool '{}' has no matching container def",
                        workspace.rel(&stage_file).display(),
                        tool_id
                    ));
                }
            }
        }
    }
    if errors.is_empty() {
        return success_line("tool/container parity: OK");
    }
    failure_block("tool/container parity check failed", errors)
}
