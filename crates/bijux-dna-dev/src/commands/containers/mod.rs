use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use chrono::{Local, NaiveDate, Utc};
use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use walkdir::WalkDir;

use crate::model::container::{ContainerCommandOutcome, NativeContainerCommandKey};
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

mod command_support;
mod dispatch;
mod metadata;
mod runtime;
mod validation;
mod versioning;

use self::command_support::*;
use self::runtime::*;

pub fn run_native_container_command(
    key: &NativeContainerCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<ContainerCommandOutcome> {
    dispatch::run_native_container_command(key, workspace, args)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_container_label_keys() -> [&'static str; 7] {
    [
        "org.opencontainers.image.source",
        "org.opencontainers.image.revision",
        "org.opencontainers.image.created",
        "org.opencontainers.image.licenses",
        "org.opencontainers.image.version",
        "org.opencontainers.image.tool",
        "org.opencontainers.image.title",
    ]
}

fn missing_container_label_markers(text: &str) -> Vec<&'static str> {
    canonical_container_label_keys()
        .into_iter()
        .filter(|label| !text.contains(label))
        .collect()
}

fn docker_image_labels(workspace: &Workspace, image: &str) -> Result<BTreeMap<String, String>> {
    let inspect = run_program_with_env(
        workspace,
        "docker",
        &[
            "image".to_string(),
            "inspect".to_string(),
            image.to_string(),
            "--format".to_string(),
            "{{json .Config.Labels}}".to_string(),
        ],
        &[],
    )?;
    if !inspect.is_success() {
        return Err(anyhow!(
            "docker image inspect failed for {image}: {}",
            inspect.stderr.trim()
        ));
    }
    let stdout = inspect.stdout.trim();
    if stdout.is_empty() || stdout == "null" {
        return Ok(BTreeMap::new());
    }
    serde_json::from_str(stdout).with_context(|| format!("parse docker labels for {image}"))
}

fn canonical_metadata_labels(labels: &BTreeMap<String, String>) -> BTreeMap<&'static str, String> {
    canonical_container_label_keys()
        .into_iter()
        .filter_map(|key| labels.get(key).cloned().map(|value| (key, value)))
        .collect()
}

fn load_toml(path: &std::path::Path) -> Result<toml::Value> {
    toml::from_str(&read_utf8(path)?).with_context(|| format!("parse TOML {}", path.display()))
}

fn registry_tool_rows(workspace: &Workspace) -> Result<Vec<toml::map::Map<String, toml::Value>>> {
    let mut rows = Vec::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let value = load_toml(&workspace.path(rel))?;
        if let Some(entries) = value.get("tools").and_then(toml::Value::as_array) {
            for entry in entries {
                if let Some(table) = entry.as_table() {
                    rows.push(table.clone());
                }
            }
        }
    }
    Ok(rows)
}

fn registry_tool_map(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let mut rows = BTreeMap::new();
    for row in registry_tool_rows(workspace)? {
        let tool_id = row
            .get("id")
            .or_else(|| row.get("tool_id"))
            .and_then(toml::Value::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if !tool_id.is_empty() {
            rows.insert(tool_id, row);
        }
    }
    Ok(rows)
}

fn governed_container_file_ids(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(workspace.path("containers/docker/arm64"))
        .with_context(|| {
            format!(
                "read {}",
                workspace.path("containers/docker/arm64").display()
            )
        })?
        .filter_map(std::result::Result::ok)
    {
        if let Some(tool_id) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.strip_prefix("Dockerfile."))
        {
            ids.insert(tool_id.to_string());
        }
    }
    for entry in fs::read_dir(workspace.path("containers/apptainer/shared"))
        .with_context(|| {
            format!(
                "read {}",
                workspace.path("containers/apptainer/shared").display()
            )
        })?
        .filter_map(std::result::Result::ok)
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("def") {
            if let Some(tool_id) = path.file_stem().and_then(|name| name.to_str()) {
                ids.insert(tool_id.to_string());
            }
        }
    }
    Ok(ids)
}

fn governed_container_statuses(workspace: &Workspace) -> Result<BTreeMap<String, String>> {
    let registry = registry_tool_map(workspace)?;
    let versions = tool_versions(workspace)?;
    let mut statuses = BTreeMap::new();
    for tool_id in governed_container_file_ids(workspace)? {
        let status = registry
            .get(&tool_id)
            .map(|row| table_string(row, "status"))
            .filter(|value| !value.is_empty())
            .or_else(|| {
                versions
                    .get(&tool_id)
                    .map(|row| table_string(row, "status"))
                    .filter(|value| !value.is_empty())
            })
            .unwrap_or_else(|| "experimental".to_string());
        statuses.insert(tool_id, status);
    }
    for (tool_id, row) in registry {
        let status = table_string(&row, "status");
        if !status.is_empty() {
            statuses.entry(tool_id).or_insert(status);
        }
    }
    Ok(statuses)
}

fn is_non_bijux_apptainer_source(workspace: &Workspace, tool_id: &str) -> bool {
    let apptainer = workspace.path(&format!("containers/apptainer/shared/{tool_id}.def"));
    apptainer.exists()
        && (read_utf8(&apptainer)
            .unwrap_or_default()
            .contains("NON_BIJUX_SOURCES.md")
            || matches!(
                tool_id,
                "bcftools"
                    | "beagle"
                    | "eagle"
                    | "eigensoft"
                    | "germline"
                    | "glimpse"
                    | "ibdhap"
                    | "ibdne"
                    | "impute5"
                    | "minimac4"
                    | "shapeit5"
            ))
}

fn tool_versions(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let value = load_toml(&workspace.path("containers/versions/versions.toml"))?;
    let Some(table) = value.as_table() else {
        return Ok(BTreeMap::new());
    };
    let mut rows = BTreeMap::new();
    for (tool, entry) in table {
        if let Some(entry_table) = entry.as_table() {
            rows.insert(tool.clone(), entry_table.clone());
        }
    }
    Ok(rows)
}

fn versions_toml_path(workspace: &Workspace) -> PathBuf {
    workspace.path("containers/versions/versions.toml")
}

fn container_version_deprecations_path(workspace: &Workspace) -> PathBuf {
    workspace.path("containers/versions/deprecations.toml")
}

fn registry_deprecations_path(workspace: &Workspace) -> PathBuf {
    workspace.path("configs/ci/registry/deprecations.toml")
}

fn lock_json_path(workspace: &Workspace) -> PathBuf {
    workspace.path("containers/versions/lock.json")
}

fn production_registry_paths(workspace: &Workspace) -> Vec<PathBuf> {
    vec![
        workspace.path("configs/ci/registry/tool_registry.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml"),
    ]
}

fn all_registry_paths(workspace: &Workspace) -> Vec<PathBuf> {
    vec![
        workspace.path("configs/ci/registry/tool_registry.toml"),
        workspace.path("configs/ci/registry/tool_registry_experimental.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml"),
    ]
}

fn read_lock_json(workspace: &Workspace) -> Result<serde_json::Value> {
    serde_json::from_str(&read_utf8(&lock_json_path(workspace))?)
        .with_context(|| "parse lock.json".to_string())
}

fn lock_items_by_tool(workspace: &Workspace) -> Result<BTreeMap<String, serde_json::Value>> {
    let mut rows = BTreeMap::new();
    if let Some(items) = read_lock_json(workspace)?
        .get("items")
        .and_then(serde_json::Value::as_array)
    {
        for row in items {
            let tool = row
                .get("tool")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !tool.is_empty() {
                rows.insert(tool, row.clone());
            }
        }
    }
    Ok(rows)
}

fn parse_date(value: &str, field_name: &str) -> Result<NaiveDate> {
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .with_context(|| format!("invalid {field_name}: {value}"))
}

fn update_status_in_table_file(
    path: &std::path::Path,
    tool: &str,
    to_status: &str,
) -> Result<bool> {
    let text = read_utf8(path)?;
    let mut updated = false;
    let mut out = Vec::new();
    let chunks = text.split("[[tools]]").collect::<Vec<_>>();
    if let Some(head) = chunks.first() {
        out.push((*head).to_string());
    }
    for chunk in chunks.iter().skip(1) {
        let mut block = format!("[[tools]]{chunk}");
        if block.contains(&format!("id = \"{tool}\""))
            || block.contains(&format!("tool_id = \"{tool}\""))
        {
            let mut lines = block.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
            if let Some(index) = lines
                .iter()
                .position(|line| line.trim_start().starts_with("status = "))
            {
                lines[index] = format!("status = \"{to_status}\"");
                updated = true;
            }
            block = format!("{}\n", lines.join("\n"));
        }
        out.push(block);
    }
    write_utf8(path, &out.concat())?;
    Ok(updated)
}

fn set_registry_status(paths: &[PathBuf], tool: &str, to_status: &str) -> Result<()> {
    let mut updated_any = false;
    for path in paths {
        updated_any |= update_status_in_table_file(path, tool, to_status)?;
    }
    if !updated_any {
        return Err(anyhow!("tool not found: {tool}"));
    }
    Ok(())
}

fn set_versions_status(workspace: &Workspace, tool: &str, to_status: &str) -> Result<()> {
    let path = versions_toml_path(workspace);
    let text = read_utf8(&path)?;
    let mut updated = false;
    let mut out = Vec::new();
    let chunks = text.split('[').collect::<Vec<_>>();
    if let Some(head) = chunks.first() {
        out.push((*head).to_string());
    }
    for chunk in chunks.iter().skip(1) {
        let block = format!("[{chunk}");
        let Some(table_end) = block.find(']') else {
            out.push(block);
            continue;
        };
        let table_name = block[1..table_end].trim();
        if table_name != tool {
            out.push(block);
            continue;
        }
        let mut lines = block.lines().map(ToOwned::to_owned).collect::<Vec<_>>();
        if let Some(index) = lines
            .iter()
            .position(|line| line.trim_start().starts_with("status = "))
        {
            lines[index] = format!("status = \"{to_status}\"");
        } else {
            lines.insert(1, format!("status = \"{to_status}\""));
        }
        updated = true;
        out.push(format!("{}\n", lines.join("\n")));
    }
    if !updated {
        return Err(anyhow!("missing versions entry for {tool}"));
    }
    write_utf8(&path, &out.concat())
}

fn append_toml_table(path: &std::path::Path, content: &str, new_file_header: &str) -> Result<()> {
    let body = if path.exists() {
        format!("{}\n\n{}", read_utf8(path)?.trim_end(), content.trim_end())
    } else {
        format!("{}{}", new_file_header, content.trim_end())
    };
    write_utf8(path, &format!("{body}\n"))
}

fn apptainer_def_paths(workspace: &Workspace) -> Vec<PathBuf> {
    let mut paths = walkdir::WalkDir::new(workspace.path("containers/apptainer"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("def"))
        .collect::<Vec<_>>();
    paths.sort();
    paths
}

fn tool_status_manifest(workspace: &Workspace) -> Result<BTreeMap<String, String>> {
    let mut statuses = BTreeMap::new();
    for raw in read_utf8(&workspace.path("containers/TOOL_IDS.txt"))?.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((tool_id, status)) = line.split_once('\t') {
            statuses.insert(tool_id.to_string(), status.to_string());
        }
    }
    Ok(statuses)
}

fn images_metadata(workspace: &Workspace) -> Result<toml::map::Map<String, toml::Value>> {
    load_toml(&workspace.path("configs/ci/tools/images.toml"))?
        .as_table()
        .cloned()
        .ok_or_else(|| anyhow!("images.toml must be a TOML table"))
}

fn toolkit_bundles(
    workspace: &Workspace,
) -> Result<BTreeMap<String, toml::map::Map<String, toml::Value>>> {
    let value = load_toml(&workspace.path("configs/ci/tools/toolkit_bundles.toml"))?;
    let mut rows = BTreeMap::new();
    if let Some(table) = value.get("bundles").and_then(toml::Value::as_table) {
        for (bundle, row) in table {
            if let Some(row) = row.as_table() {
                rows.insert(bundle.clone(), row.clone());
            }
        }
    }
    Ok(rows)
}

fn docker_tool_ids(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(workspace.path("containers/docker/arm64"))
        .with_context(|| {
            format!(
                "read {}",
                workspace.path("containers/docker/arm64").display()
            )
        })?
        .filter_map(std::result::Result::ok)
    {
        if let Some(tool) = entry
            .file_name()
            .to_str()
            .and_then(|name| name.strip_prefix("Dockerfile."))
        {
            ids.insert(tool.to_string());
        }
    }
    Ok(ids)
}

fn dockerfile_paths(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(workspace.path("containers/docker/arm64"))
        .with_context(|| {
            format!(
                "read {}",
                workspace.path("containers/docker/arm64").display()
            )
        })?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("Dockerfile."))
        })
        .collect::<Vec<_>>();
    paths.sort();
    Ok(paths)
}

fn apptainer_tool_ids(workspace: &Workspace) -> BTreeSet<String> {
    apptainer_def_paths(workspace)
        .into_iter()
        .filter_map(|path| {
            path.file_stem()
                .and_then(|name| name.to_str())
                .map(ToOwned::to_owned)
        })
        .collect()
}

fn command_hostname() -> String {
    for args in [["-f"].as_slice(), [].as_slice()] {
        let mut command = std::process::Command::new("hostname");
        command.args(args);
        let Ok(output) = command.output() else {
            continue;
        };
        if output.status.success() {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !value.is_empty() {
                return value;
            }
        }
    }
    String::new()
}

fn table_string(table: &toml::map::Map<String, toml::Value>, key: &str) -> String {
    table
        .get(key)
        .map(toml_value_string)
        .unwrap_or_default()
        .trim()
        .to_string()
}

fn table_bool(table: &toml::map::Map<String, toml::Value>, key: &str) -> bool {
    table
        .get(key)
        .and_then(toml::Value::as_bool)
        .unwrap_or(false)
}

fn table_array_strings(table: &toml::map::Map<String, toml::Value>, key: &str) -> Vec<String> {
    table
        .get(key)
        .and_then(toml::Value::as_array)
        .map(|values| {
            values
                .iter()
                .map(toml_value_string)
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

fn toml_value_string(value: &toml::Value) -> String {
    match value {
        toml::Value::String(value) => value.clone(),
        toml::Value::Integer(value) => value.to_string(),
        toml::Value::Float(value) => value.to_string(),
        toml::Value::Boolean(value) => value.to_string(),
        toml::Value::Datetime(value) => value.to_string(),
        toml::Value::Array(values) => values
            .iter()
            .map(toml_value_string)
            .collect::<Vec<_>>()
            .join(","),
        toml::Value::Table(_) => String::new(),
    }
}

fn markdown_code_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}

fn has_shell_word(line: &str, word: &str) -> bool {
    line.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
        .any(|token| token == word)
}

fn line_has_network_command(line: &str) -> bool {
    let lowered = line.to_ascii_lowercase();
    lowered.contains("git clone")
        || lowered.contains("apt-get update")
        || has_shell_word(&lowered, "curl")
        || has_shell_word(&lowered, "wget")
}

#[derive(Serialize)]
struct VersionMapItem {
    tool: String,
    version: String,
    status: String,
    source: String,
    source_sha256: String,
    pinned_commit: String,
    date_pinned: String,
}
