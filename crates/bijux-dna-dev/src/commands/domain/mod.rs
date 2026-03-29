use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_core::{id_catalog, ids};
use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};
use toml::Value as TomlValue;
use walkdir::WalkDir;

use crate::model::domain::{DomainCommandOutcome, NativeDomainCommandKey};
use crate::runtime::process::ProcessRunner;
use crate::runtime::workspace::Workspace;

const DOMAIN_INDEX_REGENERATE_PREFIX: &str =
    "# Regenerate with: cargo run -p bijux-dna-dev -- domain run generate-index -- ";
const REGISTRY_LOCK_GENERATED_BY: &str = "generated_by=bijux-dna-dev domain run lock-registry";

pub fn run_native_domain_command(
    key: &NativeDomainCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<DomainCommandOutcome> {
    match key {
        NativeDomainCommandKey::CheckDefaultSettingsDocs => {
            ensure_no_args("check-default-settings-docs", args)?;
            check_default_settings_docs(workspace)
        }
        NativeDomainCommandKey::CheckDocLinks => {
            ensure_no_args("check-doc-links", args)?;
            check_doc_links(workspace)
        }
        NativeDomainCommandKey::CheckDomainIndex => {
            ensure_no_args("check-domain-index", args)?;
            check_domain_index(workspace)
        }
        NativeDomainCommandKey::CheckDomainLayout => {
            ensure_no_args("check-domain-layout", args)?;
            check_domain_layout(workspace)
        }
        NativeDomainCommandKey::CheckDomainSchema => {
            ensure_no_args("check-domain-schema", args)?;
            check_domain_schema(workspace)
        }
        NativeDomainCommandKey::CheckDomainToolMetadata => {
            ensure_no_args("check-domain-tool-metadata", args)?;
            check_domain_tool_metadata(workspace)
        }
        NativeDomainCommandKey::CheckExternalToolPolicy => {
            ensure_no_args("check-external-tool-policy", args)?;
            check_external_tool_policy(workspace)
        }
        NativeDomainCommandKey::CheckFixtureContracts => {
            ensure_no_args("check-fixture-contracts", args)?;
            check_fixture_contracts(workspace)
        }
        NativeDomainCommandKey::CheckInventory => {
            ensure_no_args("check-inventory", args)?;
            check_inventory(workspace)
        }
        NativeDomainCommandKey::CheckOrphanFiles => {
            ensure_no_args("check-orphan-files", args)?;
            check_orphan_files(workspace)
        }
        NativeDomainCommandKey::CheckPlannerFixtureCoverage => {
            ensure_no_args("check-planner-fixture-coverage", args)?;
            check_planner_fixture_coverage(workspace)
        }
        NativeDomainCommandKey::CheckPlannerStageCoverage => {
            ensure_no_args("check-planner-stage-coverage", args)?;
            check_planner_stage_coverage(workspace)
        }
        NativeDomainCommandKey::CheckReferenceBundleLock => {
            ensure_no_args("check-reference-bundle-lock", args)?;
            check_reference_bundle_lock(workspace)
        }
        NativeDomainCommandKey::CheckRustStageCatalogParity => {
            ensure_no_args("check-rust-stage-catalog-parity", args)?;
            check_rust_stage_catalog_parity(workspace)
        }
        NativeDomainCommandKey::CheckSharedTools => {
            ensure_no_args("check-shared-tools", args)?;
            check_shared_tools(workspace)
        }
        NativeDomainCommandKey::CheckSsotAuthority => {
            ensure_no_args("check-ssot-authority", args)?;
            check_ssot_authority(workspace)
        }
        NativeDomainCommandKey::CheckToolContainerParity => {
            ensure_no_args("check-tool-container-parity", args)?;
            check_tool_container_parity(workspace)
        }
        NativeDomainCommandKey::GenerateIndex => generate_index(workspace, args),
        NativeDomainCommandKey::GenerateInventory => generate_inventory(workspace, args),
        NativeDomainCommandKey::InventoryDrift => {
            ensure_no_args("inventory-drift", args)?;
            inventory_drift(workspace)
        }
        NativeDomainCommandKey::LockRegistry => lock_registry(workspace, args),
        NativeDomainCommandKey::Validate => validate(workspace, args),
    }
}

fn ensure_no_args(command: &str, args: &[String]) -> Result<()> {
    if args.is_empty() {
        return Ok(());
    }
    bail!("{command} does not accept positional arguments");
}

fn success_line(line: impl Into<String>) -> Result<DomainCommandOutcome> {
    Ok(DomainCommandOutcome::success(format!("{}\n", line.into())))
}

fn failure_block(title: &str, errors: Vec<String>) -> Result<DomainCommandOutcome> {
    let mut stderr = String::new();
    stderr.push_str(title);
    stderr.push_str(":\n");
    for error in errors {
        stderr.push_str("- ");
        stderr.push_str(&error);
        stderr.push('\n');
    }
    Ok(DomainCommandOutcome::failure(stderr))
}

fn regex(pattern: &str) -> Result<Regex> {
    Regex::new(pattern).map_err(|error| anyhow!("invalid regex `{pattern}`: {error}"))
}

fn read_utf8(path: &Path) -> Result<String> {
    fs::read_to_string(path).with_context(|| format!("read {}", path.display()))
}

fn write_utf8(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::write_bytes(path, content).with_context(|| format!("write {}", path.display()))
}

fn scalar_from_text(text: &str, key: &str) -> Result<Option<String>> {
    let pattern = format!(r"(?m)^{}:\s*(.+?)\s*$", regex::escape(key));
    let re = regex(&pattern)?;
    Ok(re
        .captures(text)
        .and_then(|captures| captures.get(1))
        .map(|value| {
            let mut raw = value.as_str().trim().to_string();
            if (raw.starts_with('"') && raw.ends_with('"'))
                || (raw.starts_with('\'') && raw.ends_with('\''))
            {
                raw = raw[1..raw.len() - 1].trim().to_string();
            } else if let Some((before_comment, _)) = raw.split_once(" #") {
                raw = before_comment.trim().to_string();
            }
            raw
        }))
}

fn list_block(text: &str, key: &str) -> Result<Vec<String>> {
    let start_re = regex(&format!(r"^{}:\s*$", regex::escape(key)))?;
    let item_re = regex(r"^\s*-\s*([^\s#]+)\s*$")?;
    let top_level_re = regex(r"^[A-Za-z0-9_]+:\s*")?;
    let mut values = Vec::new();
    let mut in_block = false;
    for line in text.lines() {
        if start_re.is_match(line) {
            in_block = true;
            continue;
        }
        if !in_block {
            continue;
        }
        if let Some(captures) = item_re.captures(line) {
            if let Some(value) = captures.get(1) {
                values.push(value.as_str().trim_matches('"').to_string());
            }
            continue;
        }
        if !line.is_empty() && !line.starts_with(' ') && top_level_re.is_match(line) {
            break;
        }
    }
    Ok(values)
}

fn inline_list(text: &str, key: &str) -> Result<Vec<String>> {
    let pattern = format!(r"(?m)^{}:\s*\[(.*?)\]\s*$", regex::escape(key));
    let re = regex(&pattern)?;
    let Some(captures) = re.captures(text) else {
        return Ok(Vec::new());
    };
    let Some(body) = captures.get(1) else {
        return Ok(Vec::new());
    };
    let values = body
        .as_str()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_matches('"').trim_matches('\'').to_string())
        .collect::<Vec<_>>();
    Ok(values)
}

fn top_level_keys(text: &str) -> Result<BTreeSet<String>> {
    let re = regex(r"^([A-Za-z0-9_]+):")?;
    let mut keys = BTreeSet::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        if let Some(captures) = re.captures(line) {
            if let Some(key) = captures.get(1) {
                keys.insert(key.as_str().to_string());
            }
        }
    }
    Ok(keys)
}

fn required_fields(schema_path: &Path) -> Result<Vec<String>> {
    list_block(&read_utf8(schema_path)?, "required_fields")
}

fn allowed_payload_keys(schema_path: &Path) -> Result<Vec<String>> {
    list_block(&read_utf8(schema_path)?, "allowed_payload_keys")
}

fn declared_stage_key(path: &Path) -> Result<Option<String>> {
    let Some(stage_id) = scalar_from_text(&read_utf8(path)?, "stage_id")? else {
        return Ok(None);
    };
    let stage_id = ids::parse_stage_id(&stage_id)
        .map_err(|err| anyhow!("{}: invalid stage_id {stage_id:?}: {err}", path.display()))?;
    Ok(Some(stage_id.as_str().to_string()))
}

fn declared_tool_key(path: &Path) -> Result<Option<String>> {
    let Some(tool_id) = scalar_from_text(&read_utf8(path)?, "tool_id")? else {
        return Ok(None);
    };
    let tool_id = ids::parse_tool_id(&tool_id)
        .map_err(|err| anyhow!("{}: invalid tool_id {tool_id:?}: {err}", path.display()))?;
    Ok(Some(tool_id.as_str().to_string()))
}

fn parse_status(path: &Path) -> Result<Option<String>> {
    scalar_from_text(&read_utf8(path)?, "status")
}

fn domain_directories(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let mut directories = fs::read_dir(workspace.path("domain"))
        .with_context(|| format!("read {}", workspace.path("domain").display()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| match entry.file_type() {
            Ok(file_type) if file_type.is_dir() => Some(entry.path()),
            _ => None,
        })
        .collect::<Vec<_>>();
    directories.sort();
    Ok(directories)
}

fn yaml_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(dir)
        .with_context(|| format!("read {}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| match entry.file_type() {
            Ok(file_type)
                if file_type.is_file()
                    && entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml") =>
            {
                Some(entry.path())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn markdown_files(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = fs::read_dir(dir)
        .with_context(|| format!("read {}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| match entry.file_type() {
            Ok(file_type)
                if file_type.is_file()
                    && entry.path().extension().and_then(|ext| ext.to_str()) == Some("md") =>
            {
                Some(entry.path())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn render_domain_index(workspace: &Workspace, dom: &str) -> Result<String> {
    let dom_dir = workspace.path(&format!("domain/{dom}"));
    let index_path = dom_dir.join("index.yaml");
    if !index_path.is_file() {
        bail!("missing {}", index_path.display());
    }

    let mut stage_ids = BTreeSet::new();
    let mut governed_stage_ids = BTreeSet::new();
    for stage_file in yaml_files(&dom_dir.join("stages"))? {
        if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        if let Some(stage_id) = declared_stage_key(&stage_file)? {
            if parse_status(&stage_file)?.as_deref() == Some("supported") {
                governed_stage_ids.insert(stage_id.clone());
            }
            stage_ids.insert(stage_id);
        }
    }

    let mut tool_ids = BTreeSet::new();
    let mut governed_tool_ids = BTreeSet::new();
    for tool_file in yaml_files(&dom_dir.join("tools"))? {
        if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        if let Some(tool_id) = declared_tool_key(&tool_file)? {
            if parse_status(&tool_file)?.as_deref() == Some("supported") {
                governed_tool_ids.insert(tool_id.clone());
            }
            tool_ids.insert(tool_id);
        }
    }

    let existing = read_utf8(&index_path)?;
    let mut lines = existing
        .lines()
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    if lines
        .first()
        .is_some_and(|line| line == "# GENERATED FILE - DO NOT EDIT")
    {
        lines.remove(0);
        if lines
            .first()
            .is_some_and(|line| line.starts_with("# Regenerate with:"))
        {
            lines.remove(0);
        }
        while lines.first().is_some_and(|line| line.trim().is_empty()) {
            lines.remove(0);
        }
    }
    let mut body_lines = lines;

    replace_or_insert_block(
        &mut body_lines,
        "stage_ids",
        stage_ids
            .iter()
            .map(|stage_id| format!("  - {stage_id}"))
            .collect(),
        Some("domain_version"),
    )?;
    replace_or_insert_block(
        &mut body_lines,
        "tool_ids",
        tool_ids
            .iter()
            .map(|tool_id| format!("  - {tool_id}"))
            .collect(),
        Some("stage_ids"),
    )?;
    replace_or_insert_block(
        &mut body_lines,
        "governed_stage_ids",
        governed_stage_ids
            .iter()
            .map(|stage_id| format!("  - {stage_id}"))
            .collect(),
        Some("tool_ids"),
    )?;
    replace_or_insert_block(
        &mut body_lines,
        "governed_tool_ids",
        governed_tool_ids
            .iter()
            .map(|tool_id| format!("  - {tool_id}"))
            .collect(),
        Some("governed_stage_ids"),
    )?;
    replace_block(
        &mut body_lines,
        "stage_tool_compatibility",
        render_stage_tool_compatibility_block(&dom_dir)?,
    )?;
    replace_or_insert_block(
        &mut body_lines,
        "stage_tool_integration",
        render_stage_tool_integration_block(&dom_dir)?,
        Some("stage_tool_compatibility"),
    )?;
    replace_or_insert_block(
        &mut body_lines,
        "reference_index_compatibility",
        render_reference_index_compatibility_block(&dom_dir)?,
        Some("stage_tool_integration"),
    )?;

    if !body_lines
        .iter()
        .any(|line| line.starts_with("domain_version:"))
    {
        let Some(domain_line_index) = body_lines
            .iter()
            .position(|line| line.starts_with("domain:"))
        else {
            bail!("{}: missing domain: field", index_path.display());
        };
        let version = if dom == "vcf" { "v2" } else { "v1" };
        body_lines.insert(domain_line_index + 1, format!("domain_version: {version}"));
    }

    let header = [
        "# GENERATED FILE - DO NOT EDIT".to_string(),
        format!("{DOMAIN_INDEX_REGENERATE_PREFIX}{dom}"),
        String::new(),
    ];
    Ok(format!(
        "{}\n",
        header
            .into_iter()
            .chain(body_lines)
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

fn render_stage_tool_compatibility_block(dom_dir: &Path) -> Result<Vec<String>> {
    let mut rendered = Vec::new();
    let mut stage_map = BTreeMap::<String, Vec<String>>::new();
    for stage_file in yaml_files(&dom_dir.join("stages"))? {
        if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let text = read_utf8(&stage_file)?;
        let Some(stage_id) = scalar_from_text(&text, "stage_id")? else {
            continue;
        };
        let compatible = {
            let block = list_block(&text, "compatible_tools")?;
            if block.is_empty() {
                inline_list(&text, "compatible_tools")?
            } else {
                block
            }
        };
        stage_map.insert(stage_id, compatible);
    }

    for (stage_id, tools) in stage_map {
        rendered.push(format!("  {stage_id}:"));
        rendered.extend(tools.into_iter().map(|tool_id| format!("  - {tool_id}")));
    }
    Ok(rendered)
}

fn render_stage_tool_integration_block(dom_dir: &Path) -> Result<Vec<String>> {
    let mut rendered = Vec::new();
    let mut stage_map = BTreeMap::<String, BTreeMap<String, String>>::new();
    for stage_file in yaml_files(&dom_dir.join("stages"))? {
        if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let text = read_utf8(&stage_file)?;
        let Some(stage_id) = scalar_from_text(&text, "stage_id")? else {
            continue;
        };
        let mut integration = BTreeMap::new();
        let compatible = {
            let block = list_block(&text, "compatible_tools")?;
            if block.is_empty() {
                inline_list(&text, "compatible_tools")?
            } else {
                block
            }
        };
        for tool_id in compatible {
            integration.insert(tool_id, "governed_contract".to_string());
        }
        let planned = {
            let block = list_block(&text, "planned_out_of_scope")?;
            if block.is_empty() {
                inline_list(&text, "planned_out_of_scope")?
            } else {
                block
            }
        };
        for tool_id in planned {
            integration.insert(tool_id, "planned_contract".to_string());
        }
        stage_map.insert(stage_id, integration);
    }

    for (stage_id, tool_map) in stage_map {
        rendered.push(format!("  {stage_id}:"));
        for (tool_id, level) in tool_map {
            rendered.push(format!("    {tool_id}: {level}"));
        }
    }
    Ok(rendered)
}

fn render_reference_index_compatibility_block(dom_dir: &Path) -> Result<Vec<String>> {
    let mut rendered = Vec::new();
    let mut tool_map = BTreeMap::<String, Vec<String>>::new();
    for tool_file in yaml_files(&dom_dir.join("tools"))? {
        if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
            continue;
        }
        let text = read_utf8(&tool_file)?;
        let Some(tool_id) = scalar_from_text(&text, "tool_id")? else {
            continue;
        };
        let backends = {
            let block = list_block(&text, "reference_index_backends")?;
            if block.is_empty() {
                inline_list(&text, "reference_index_backends")?
            } else {
                block
            }
        };
        if !backends.is_empty() {
            tool_map.insert(tool_id, backends);
        }
    }

    for (tool_id, backends) in tool_map {
        rendered.push(format!("  {tool_id}:"));
        rendered.extend(backends.into_iter().map(|backend| format!("  - {backend}")));
    }
    Ok(rendered)
}

fn replace_block(lines: &mut Vec<String>, key: &str, items: Vec<String>) -> Result<()> {
    let start_re = regex(&format!(r"^{}:\s*$", regex::escape(key)))?;
    let top_level_re = regex(r"^[A-Za-z0-9_]+:\s*")?;
    let Some(start) = lines.iter().position(|line| start_re.is_match(line)) else {
        bail!("missing {key}: block");
    };
    let mut end = lines.len();
    for (index, line) in lines.iter().enumerate().skip(start + 1) {
        if !line.is_empty() && !line.starts_with(' ') && top_level_re.is_match(line) {
            end = index;
            break;
        }
    }
    let mut replacement = vec![format!("{key}:")];
    replacement.extend(items);
    lines.splice(start..end, replacement);
    Ok(())
}

fn replace_or_insert_block(
    lines: &mut Vec<String>,
    key: &str,
    items: Vec<String>,
    after_key: Option<&str>,
) -> Result<()> {
    if lines.iter().any(|line| line == &format!("{key}:")) {
        return replace_block(lines, key, items);
    }
    let mut replacement = vec![format!("{key}:")];
    replacement.extend(items);
    let insert_at = if let Some(after_key) = after_key {
        let start_re = regex(&format!(r"^{}:\s*$", regex::escape(after_key)))?;
        let top_level_re = regex(r"^[A-Za-z0-9_]+:\s*")?;
        let Some(start) = lines.iter().position(|line| start_re.is_match(line)) else {
            bail!("missing {after_key}: block");
        };
        let mut end = lines.len();
        for (index, line) in lines.iter().enumerate().skip(start + 1) {
            if !line.is_empty() && !line.starts_with(' ') && top_level_re.is_match(line) {
                end = index;
                break;
            }
        }
        end
    } else {
        lines.len()
    };
    lines.splice(insert_at..insert_at, replacement);
    Ok(())
}

fn command_runner(workspace: &Workspace) -> ProcessRunner<'_> {
    ProcessRunner::new(workspace)
}

fn artifact_env(workspace: &Workspace) -> Result<Vec<(String, String)>> {
    let artifact_root = workspace.path("artifacts");
    bijux_dna_infra::ensure_dir(&artifact_root)
        .with_context(|| format!("create {}", artifact_root.display()))?;
    let cargo_target_dir = artifact_root.join("target");
    let cargo_home = artifact_root.join("cargo/home");
    let tmpdir = artifact_root.join("tmp");
    bijux_dna_infra::ensure_dir(&cargo_target_dir)
        .with_context(|| format!("create {}", cargo_target_dir.display()))?;
    bijux_dna_infra::ensure_dir(&cargo_home)
        .with_context(|| format!("create {}", cargo_home.display()))?;
    bijux_dna_infra::ensure_dir(&tmpdir).with_context(|| format!("create {}", tmpdir.display()))?;
    Ok(vec![
        (
            "ARTIFACT_ROOT".to_string(),
            artifact_root.display().to_string(),
        ),
        ("ISO_ROOT".to_string(), artifact_root.display().to_string()),
        (
            "CARGO_TARGET_DIR".to_string(),
            cargo_target_dir.display().to_string(),
        ),
        ("CARGO_HOME".to_string(), cargo_home.display().to_string()),
        ("TMPDIR".to_string(), tmpdir.display().to_string()),
        ("TMP".to_string(), tmpdir.display().to_string()),
        ("TEMP".to_string(), tmpdir.display().to_string()),
        ("TZ".to_string(), "UTC".to_string()),
        ("LC_ALL".to_string(), "C".to_string()),
    ])
}

fn load_toml(path: &Path) -> Result<TomlValue> {
    toml::from_str(&read_utf8(path)?).with_context(|| format!("parse TOML {}", path.display()))
}

fn toml_tools(path: &Path) -> Result<Vec<TomlValue>> {
    Ok(load_toml(path)?
        .get("tools")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default())
}

fn toml_stages(path: &Path) -> Result<Vec<TomlValue>> {
    Ok(load_toml(path)?
        .get("stages")
        .and_then(TomlValue::as_array)
        .cloned()
        .unwrap_or_default())
}

fn tool_registry_files(workspace: &Workspace) -> Vec<PathBuf> {
    vec![
        workspace.path("configs/ci/registry/tool_registry.toml"),
        workspace.path("configs/ci/registry/tool_registry_experimental.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf.toml"),
        workspace.path("configs/ci/registry/tool_registry_vcf_downstream.toml"),
    ]
}

fn cargo_registry_list_tools(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let output = command_runner(workspace).run_owned_with_env(
        "cargo",
        &[
            "run".to_string(),
            "--quiet".to_string(),
            "--bin".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "registry".to_string(),
            "list-tools".to_string(),
        ],
        &artifact_env(workspace)?,
    )?;
    if !output.status.success() {
        bail!("cargo registry list-tools failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn cargo_registry_list_stages(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let output = command_runner(workspace).run_owned_with_env(
        "cargo",
        &[
            "run".to_string(),
            "--quiet".to_string(),
            "--bin".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "registry".to_string(),
            "list-stages".to_string(),
        ],
        &artifact_env(workspace)?,
    )?;
    if !output.status.success() {
        bail!("cargo registry list-stages failed");
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split(',')
        .flat_map(|chunk| chunk.lines())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn cargo_registry_stage_tools(workspace: &Workspace, stage_id: &str) -> Result<BTreeSet<String>> {
    let output = command_runner(workspace).run_owned_with_env(
        "cargo",
        &[
            "run".to_string(),
            "--quiet".to_string(),
            "--bin".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "registry".to_string(),
            "list-tools".to_string(),
            "--stage".to_string(),
            stage_id.to_string(),
            "--kind".to_string(),
            "all".to_string(),
        ],
        &artifact_env(workspace)?,
    )?;
    if !output.status.success() {
        return Ok(BTreeSet::new());
    }
    Ok(String::from_utf8_lossy(&output.stdout)
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .collect())
}

fn check_default_settings_docs(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    let required_sections = ["inputs", "outputs", "key parameters", "validity limits"];
    let stage_re = regex(r#"(?m)^stage_id:\s*"?([^"\n#]+)"?\s*$"#)?;
    let stage_line_re = regex(r"(?m)^\s{2}([a-z0-9._-]+):\s*(.*)$")?;
    let nested_item_re = regex(r"^\s{4}-\s*([a-z0-9._-]+)\s*$")?;
    let top_level_re = regex(r"^[A-Za-z0-9_]+:\s*")?;

    for dom_dir in domain_directories(workspace)? {
        let dom = dom_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid domain directory {}", dom_dir.display()))?;
        let doc = dom_dir.join("docs/DEFAULT_SETTINGS.md");
        if !doc.is_file() {
            errors.push(format!("domain/{dom}/docs/DEFAULT_SETTINGS.md missing"));
            continue;
        }
        let text = read_utf8(&doc)?.to_lowercase();
        for section in required_sections {
            if !text.contains(section) {
                errors.push(format!(
                    "{}: missing required section phrase '{section}'",
                    workspace.rel(&doc).display()
                ));
            }
        }

        let mut stage_ids = Vec::new();
        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let stage_text = read_utf8(&stage_file)?;
            if let Some(captures) = stage_re.captures(&stage_text) {
                if let Some(stage_id) = captures.get(1) {
                    stage_ids.push(stage_id.as_str().trim().to_string());
                }
            }
        }

        let idx = dom_dir.join("index.yaml");
        let idx_text = if idx.is_file() {
            read_utf8(&idx)?
        } else {
            String::new()
        };
        let active_default_start = idx_text
            .lines()
            .position(|line| line.starts_with("active_default_rationale:"));

        for stage in stage_ids {
            let stage_lower = stage.to_lowercase();
            if !text.contains(&stage_lower) {
                errors.push(format!(
                    "{}: missing stage coverage for '{stage}'",
                    workspace.rel(&doc).display()
                ));
            }
            if !regex(&format!(r"{}.*default", regex::escape(&stage_lower)))?.is_match(&text) {
                errors.push(format!(
                    "{}: missing blessed default description for '{stage}'",
                    workspace.rel(&doc).display()
                ));
            }
            let has_doc_rationale =
                regex(&format!(r"{}.*rationale", regex::escape(&stage_lower)))?.is_match(&text);
            let has_idx_default = regex(&format!(r"(?m)^\s{{2}}{}:\s*.+$", regex::escape(&stage)))?
                .is_match(&idx_text);

            let mut has_idx_rationale = false;
            if let Some(start_index) = active_default_start {
                for line in idx_text.lines().skip(start_index + 1) {
                    if top_level_re.is_match(line) {
                        break;
                    }
                    if regex(&format!(r"^\s{{2}}{}:\s*.+$", regex::escape(&stage)))?.is_match(line)
                    {
                        has_idx_rationale = true;
                        break;
                    }
                }
            }

            if !(has_doc_rationale || has_idx_rationale) {
                errors.push(format!(
                    "{}: missing blessed default rationale for '{stage}'",
                    workspace.rel(&doc).display()
                ));
            }
            if !(has_idx_default
                || regex(&format!(r"{}.*default", regex::escape(&stage_lower)))?.is_match(&text))
            {
                errors.push(format!(
                    "{}: missing blessed default mapping for '{stage}'",
                    workspace.rel(&doc).display()
                ));
            }
        }

        if idx.is_file() {
            let mut in_block = false;
            let mut mapping = BTreeMap::<String, Vec<String>>::new();
            let mut current_stage = None::<String>;
            for line in idx_text.lines() {
                if line.starts_with("stage_tool_compatibility:") {
                    in_block = true;
                    continue;
                }
                if in_block && top_level_re.is_match(line) {
                    break;
                }
                if !in_block {
                    continue;
                }
                if let Some(captures) = stage_line_re.captures(line) {
                    let stage = captures
                        .get(1)
                        .map(|value| value.as_str().to_string())
                        .ok_or_else(|| anyhow!("missing stage_tool_compatibility key"))?;
                    let rest = captures
                        .get(2)
                        .map(|value| value.as_str().trim().to_string())
                        .unwrap_or_default();
                    let tools = if rest.starts_with('[') && rest.ends_with(']') {
                        rest.trim_matches(|ch| ch == '[' || ch == ']')
                            .split(',')
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(|value| value.trim_matches('"').to_string())
                            .collect::<Vec<_>>()
                    } else {
                        Vec::new()
                    };
                    mapping.insert(stage.clone(), tools);
                    current_stage = Some(stage);
                    continue;
                }
                if let Some(captures) = nested_item_re.captures(line) {
                    if let (Some(stage), Some(tool)) = (current_stage.clone(), captures.get(1)) {
                        mapping
                            .entry(stage)
                            .or_default()
                            .push(tool.as_str().to_string());
                    }
                }
            }
            for (stage, tools) in mapping {
                if tools.len() == 1 {
                    let marker = format!("single_tool_justification: {stage}").to_lowercase();
                    if !text.contains(&marker) {
                        errors.push(format!(
                            "{}: missing '{marker}' for single-tool stage",
                            workspace.rel(&doc).display()
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        return success_line("default-settings docs: OK");
    }
    failure_block("default-settings docs check failed", errors)
}

fn check_doc_links(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    let link_re = regex(r"\[[^\]]*\]\(([^)]+)\)")?;
    for dom_dir in domain_directories(workspace)? {
        let docs_dir = dom_dir.join("docs");
        if !docs_dir.is_dir() {
            continue;
        }
        for md in markdown_files(&docs_dir)? {
            let text = read_utf8(&md)?;
            for captures in link_re.captures_iter(&text) {
                let Some(target_match) = captures.get(1) else {
                    continue;
                };
                let target = target_match.as_str().trim();
                if target.is_empty()
                    || target.starts_with("http://")
                    || target.starts_with("https://")
                    || target.starts_with("mailto:")
                    || target.starts_with('#')
                {
                    continue;
                }
                let target_path = target.split('#').next().unwrap_or_default();
                let candidate = md
                    .parent()
                    .ok_or_else(|| anyhow!("missing parent for {}", md.display()))?
                    .join(target_path);
                if !candidate.exists() {
                    errors.push(format!("{} -> {target}", workspace.rel(&md).display()));
                }
            }
        }
    }
    if errors.is_empty() {
        return success_line("domain docs links: OK");
    }
    failure_block("domain docs link check failed", errors)
}

fn check_domain_index(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        let dom = dom_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid domain directory {}", dom_dir.display()))?;
        let index_path = dom_dir.join("index.yaml");
        if !index_path.is_file() {
            continue;
        }
        let actual = read_utf8(&index_path)?;
        let mut actual_lines = actual.lines();
        if actual_lines.next() != Some("# GENERATED FILE - DO NOT EDIT") {
            errors.push(format!(
                "domain index: missing generated header in domain/{dom}/index.yaml"
            ));
        }
        if actual_lines
            .next()
            .is_none_or(|line| line != format!("{DOMAIN_INDEX_REGENERATE_PREFIX}{dom}"))
        {
            errors.push(format!(
                "domain index: missing regenerate header in domain/{dom}/index.yaml"
            ));
        }

        let expected = render_domain_index(workspace, dom)?;
        if expected != actual {
            errors.push(format!(
                "domain index drift for domain/{dom}/index.yaml; regenerate with cargo run -p bijux-dna-dev -- domain run generate-index -- {dom}"
            ));
        }

        let stage_ids = list_block(&actual, "stage_ids")?;
        let tool_ids = list_block(&actual, "tool_ids")?
            .into_iter()
            .collect::<BTreeSet<_>>();

        let mut stage_file_map = BTreeMap::<String, PathBuf>::new();
        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            if let Some(stage_id) = declared_stage_key(&stage_file)? {
                stage_file_map.insert(stage_id, stage_file);
            }
        }

        for stage_id in &stage_ids {
            if !stage_file_map.contains_key(stage_id) {
                errors.push(format!(
                    "{}: stage {stage_id} is listed but no stages/*.yaml declares it",
                    workspace.rel(&index_path).display()
                ));
                continue;
            }
            let fixture_dir = dom_dir.join("fixtures").join(stage_id);
            let has_files = fixture_dir.exists()
                && WalkDir::new(&fixture_dir)
                    .into_iter()
                    .filter_map(std::result::Result::ok)
                    .any(|entry| entry.file_type().is_file());
            if !has_files {
                errors.push(format!(
                    "{}: stage {stage_id} must have at least one fixture under {}",
                    workspace.rel(&index_path).display(),
                    workspace.rel(&fixture_dir).display()
                ));
            }
        }

        let mut declared_tools = BTreeSet::new();
        for tool_file in yaml_files(&dom_dir.join("tools"))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            if let Some(tool_id) = declared_tool_key(&tool_file)? {
                declared_tools.insert(tool_id);
            }
        }

        let fixtures_root = dom_dir.join("fixtures");
        if fixtures_root.is_dir() {
            let stage_dirs = fs::read_dir(&fixtures_root)
                .with_context(|| format!("read {}", fixtures_root.display()))?
                .filter_map(std::result::Result::ok)
                .filter_map(|entry| match entry.file_type() {
                    Ok(file_type) if file_type.is_dir() => Some(entry.path()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            for stage_dir in stage_dirs {
                for fixture in fs::read_dir(&stage_dir)
                    .with_context(|| format!("read {}", stage_dir.display()))?
                    .filter_map(std::result::Result::ok)
                {
                    if fixture.path().extension().and_then(|ext| ext.to_str()) != Some("txt") {
                        continue;
                    }
                    let tool_id = fixture
                        .path()
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .ok_or_else(|| {
                            anyhow!("invalid fixture file {}", fixture.path().display())
                        })?
                        .to_string();
                    if !declared_tools.contains(&tool_id) {
                        errors.push(format!(
                            "{}: fixture tool '{tool_id}' missing matching tools/<tool>.yaml in domain/{dom}",
                            workspace.rel(&fixture.path()).display()
                        ));
                    }
                }
            }
        }

        for tool_id in &tool_ids {
            if !declared_tools.contains(tool_id) {
                errors.push(format!(
                    "{}: tool {tool_id} listed in tool_ids but missing tools/<tool>.yaml",
                    workspace.rel(&index_path).display()
                ));
            }
        }

        for stage_id in stage_file_map.keys() {
            if !stage_ids.contains(stage_id) {
                errors.push(format!(
                    "{}: missing stage_id listing for stages file declaring '{stage_id}'",
                    workspace.rel(&index_path).display()
                ));
            }
        }
        for tool_id in &declared_tools {
            if !tool_ids.contains(tool_id) {
                errors.push(format!(
                    "{}: missing tool_id listing for tools file declaring '{tool_id}'",
                    workspace.rel(&index_path).display()
                ));
            }
        }
    }

    if errors.is_empty() {
        return success_line("domain index/completeness: OK");
    }
    failure_block("domain completeness check failed", errors)
}

fn check_domain_layout(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let domain_dir = workspace.path("domain");
    if !domain_dir.is_dir() {
        return Ok(DomainCommandOutcome {
            exit_code: 1,
            stdout: String::new(),
            stderr: format!("domain layout: missing {}\n", domain_dir.display()),
        });
    }

    let allowed = [
        regex(
            r"^domain/[^/]+/(index\.yaml|artifacts\.yaml|metrics\.yaml|execution_support\.yaml)$",
        )?,
        regex(r"^domain/[^/]+/(stages|tools)/[^/]+\.yaml$")?,
        regex(r"^domain/[^/]+/(metrics|artifacts)/_schema\.yaml$")?,
        regex(r"^domain/[^/]+/fixtures/[^/]+(?:/[^/]+){0,2}$")?,
        regex(r"^domain/[^/]+/docs/[^/]+(?:/[^/]+)?$")?,
    ];

    let mut errors = Vec::new();
    for entry in WalkDir::new(&domain_dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
        if rel.ends_with(".tmp") {
            errors.push(format!(
                "domain layout: forbidden *.tmp files under domain/\n{rel}"
            ));
            continue;
        }
        if allowed.iter().all(|pattern| !pattern.is_match(&rel)) {
            errors.push(format!(
                "domain layout: unknown file not in allowlist: {rel}"
            ));
        }
    }

    if errors.is_empty() {
        return success_line("domain layout: OK");
    }
    Ok(DomainCommandOutcome::failure(format!(
        "{}\n",
        errors.join("\n")
    )))
}

fn production_bindings(workspace: &Workspace) -> Result<BTreeSet<(String, String)>> {
    let mut bindings = BTreeSet::new();
    for row in toml_tools(&workspace.path("configs/ci/registry/tool_registry.toml"))? {
        let Some(table) = row.as_table() else {
            continue;
        };
        let tool_id = table
            .get("id")
            .or_else(|| table.get("tool_id"))
            .and_then(TomlValue::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        let status = table
            .get("status")
            .and_then(TomlValue::as_str)
            .unwrap_or_default()
            .trim()
            .to_string();
        if tool_id.is_empty() || !matches!(status.as_str(), "production" | "supported") {
            continue;
        }
        for binding in table
            .get("bindings")
            .and_then(TomlValue::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(stage_id) = binding.as_str() {
                bindings.insert((stage_id.trim().to_string(), tool_id.clone()));
            }
        }
    }
    Ok(bindings)
}

fn check_domain_schema(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    let production_bindings = production_bindings(workspace)?;
    let downstream = workspace.path("configs/ci/stages/stages_vcf_downstream.toml");
    if downstream.is_file() {
        let rows = toml_stages(&downstream)?;
        if rows.is_empty() {
            errors.push(format!(
                "{}: must define at least one [[stages]] entry",
                downstream.display()
            ));
        }
        for row in rows {
            let Some(table) = row.as_table() else {
                continue;
            };
            let stage_id = table
                .get("id")
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let status = table
                .get("status")
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if !stage_id.starts_with("vcf.") {
                errors.push(format!(
                    "{}: downstream stage id must start with 'vcf.': {stage_id}",
                    downstream.display()
                ));
            }
            if !matches!(
                status.as_str(),
                "planned" | "experimental" | "production" | "supported"
            ) {
                errors.push(format!(
                    "{}: invalid stage status '{status}' for {stage_id}",
                    downstream.display()
                ));
            }
        }
    } else {
        errors.push(format!(
            "{}: missing required downstream stages registry file",
            downstream.display()
        ));
    }

    let repeated_token_re = regex(r"__")?;
    let snake_case_re = regex(r"^[a-z0-9_]+$")?;
    let stage_slug_re = regex(r"^[a-z0-9_]+$")?;
    let metric_item_re = regex(r"(?m)^\s*-\s*[a-z0-9_]+\s*$")?;
    let metric_entry_re = regex(r#"(?m)^\s*-\s*id:\s*"?[a-z0-9_]+"?\s*$"#)?;

    for dom_dir in domain_directories(workspace)? {
        let dom = dom_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid domain directory {}", dom_dir.display()))?
            .to_string();
        let stage_schema = dom_dir.join("stages/_schema.yaml");
        let tool_schema = dom_dir.join("tools/_schema.yaml");
        if !(stage_schema.is_file() && tool_schema.is_file()) {
            continue;
        }
        let required_stage = required_fields(&stage_schema)?;
        let required_tool = required_fields(&tool_schema)?;
        let required_scope = scalar_from_text(&read_utf8(&stage_schema)?, "required_scope")?;
        let required_domain = scalar_from_text(&read_utf8(&stage_schema)?, "domain")?;
        let required_tool_scope = scalar_from_text(&read_utf8(&tool_schema)?, "required_scope")?;

        let mut stage_ids_seen = BTreeSet::new();
        let mut tool_ids_seen = BTreeSet::new();

        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&stage_file)?;
            let keys = top_level_keys(&text)?;
            let missing = required_stage
                .iter()
                .filter(|field| !keys.contains(field.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                errors.push(format!(
                    "{}: missing required fields: {:?}",
                    stage_file.display(),
                    missing
                ));
            }

            let stage_id = scalar_from_text(&text, "stage_id")?;
            if let Some(stage_id_value) = stage_id.clone() {
                if !stage_ids_seen.insert(stage_id_value.clone()) {
                    errors.push(format!(
                        "{}: duplicate stage_id in domain {dom}: {stage_id_value}",
                        stage_file.display()
                    ));
                }
                let prefix = format!("{dom}.");
                if stage_id_value.starts_with(&prefix) {
                    let slug = stage_id_value.trim_start_matches(&prefix);
                    if !stage_slug_re.is_match(slug) {
                        errors.push(format!(
                            "{}: stage slug '{slug}' must match [a-z0-9_]+",
                            stage_file.display()
                        ));
                    }
                    if repeated_token_re.is_match(slug) {
                        errors.push(format!(
                            "{}: stage slug '{slug}' must not contain '__'",
                            stage_file.display()
                        ));
                    }
                    let parts = slug
                        .split('_')
                        .filter(|part| !part.is_empty())
                        .collect::<Vec<_>>();
                    for pair in parts.windows(2) {
                        if pair[0] == pair[1] {
                            errors.push(format!(
                                "{}: stage slug '{slug}' has repeated adjacent token '{}'",
                                stage_file.display(),
                                pair[0]
                            ));
                        }
                    }
                } else {
                    errors.push(format!(
                        "{}: stage_id must use '<domain>.<stage_slug>' format",
                        stage_file.display()
                    ));
                }
            } else {
                errors.push(format!("{}: missing stage_id", stage_file.display()));
            }

            let scope = scalar_from_text(&text, "scope")?;
            if required_scope
                .as_deref()
                .is_some_and(|required| scope.as_deref() != Some(required))
            {
                errors.push(format!(
                    "{}: scope must be {} (got {})",
                    stage_file.display(),
                    required_scope.clone().unwrap_or_default(),
                    scope.unwrap_or_default()
                ));
            }
            let declared_domain = scalar_from_text(&text, "domain")?;
            if required_domain
                .as_deref()
                .is_some_and(|required| declared_domain.as_deref() != Some(required))
            {
                errors.push(format!(
                    "{}: domain must be {} (got {})",
                    stage_file.display(),
                    required_domain.clone().unwrap_or_default(),
                    declared_domain.unwrap_or_default()
                ));
            }
            let defaults_source = scalar_from_text(&text, "defaults_source")?;
            match defaults_source {
                Some(value) if value.starts_with("citation:") || value.starts_with("doc_ref:") => {}
                Some(value) => errors.push(format!(
                    "{}: defaults_source must start with citation: or doc_ref: (got {value})",
                    stage_file.display()
                )),
                None => errors.push(format!("{}: missing defaults_source", stage_file.display())),
            }

            if dom == "vcf" {
                let compatible = inline_list(&text, "compatible_tools")?;
                let single_justification = scalar_from_text(&text, "single_tool_justification")?;
                if compatible.len() < 2 && single_justification.is_none() {
                    errors.push(format!(
                        "{}: single-tool stage requires single_tool_justification when compatible_tools has <2 tools",
                        stage_file.display()
                    ));
                }
            }
        }

        for tool_file in yaml_files(&dom_dir.join("tools"))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&tool_file)?;
            let keys = top_level_keys(&text)?;
            let missing = required_tool
                .iter()
                .filter(|field| !keys.contains(field.as_str()))
                .cloned()
                .collect::<Vec<_>>();
            if !missing.is_empty() {
                errors.push(format!(
                    "{}: missing required fields: {:?}",
                    tool_file.display(),
                    missing
                ));
            }
            let tool_id = scalar_from_text(&text, "tool_id")?;
            if let Some(tool_id_value) = tool_id.clone() {
                if !tool_ids_seen.insert(tool_id_value.clone()) {
                    errors.push(format!(
                        "{}: duplicate tool_id in domain {dom}: {tool_id_value}",
                        tool_file.display()
                    ));
                }
                if !snake_case_re.is_match(&tool_id_value) {
                    errors.push(format!(
                        "{}: tool_id '{tool_id_value}' must be snake_case ([a-z0-9_]+)",
                        tool_file.display()
                    ));
                }
            } else {
                errors.push(format!("{}: missing tool_id", tool_file.display()));
            }
            let scope = scalar_from_text(&text, "scope")?;
            if required_tool_scope
                .as_deref()
                .is_some_and(|required| scope.as_deref() != Some(required))
            {
                errors.push(format!(
                    "{}: scope must be {} (got {})",
                    tool_file.display(),
                    required_tool_scope.clone().unwrap_or_default(),
                    scope.unwrap_or_default()
                ));
            }
        }

        let metrics_file = dom_dir.join("metrics.yaml");
        let metrics_schema = dom_dir.join("metrics/_schema.yaml");
        if !metrics_schema.is_file() {
            errors.push(format!(
                "{}: missing metrics schema {}",
                dom_dir.display(),
                workspace.rel(&metrics_schema).display()
            ));
        }
        if metrics_file.is_file() {
            let text = read_utf8(&metrics_file)?;
            let keys = top_level_keys(&text)?;
            if metrics_schema.is_file() {
                let required = required_fields(&metrics_schema)?;
                let missing = required
                    .iter()
                    .filter(|field| !keys.contains(field.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();
                if !missing.is_empty() {
                    errors.push(format!(
                        "{}: missing required fields from schema: {:?}",
                        metrics_file.display(),
                        missing
                    ));
                }
                let payload_keys = allowed_payload_keys(&metrics_schema)?;
                if !payload_keys.is_empty() && !payload_keys.iter().any(|key| keys.contains(key)) {
                    errors.push(format!(
                        "{}: must define at least one payload key from {:?}",
                        metrics_file.display(),
                        payload_keys
                    ));
                }
            }
            let schema_version = scalar_from_text(&text, "schema_version")?;
            if !schema_version
                .as_deref()
                .is_some_and(|value| value.starts_with("bijux."))
            {
                errors.push(format!(
                    "{}: schema_version must exist and start with 'bijux.'",
                    metrics_file.display()
                ));
            }
            if scalar_from_text(&text, "domain")?.as_deref() != Some(dom.as_str()) {
                errors.push(format!(
                    "{}: domain must be '{dom}' (got {})",
                    metrics_file.display(),
                    scalar_from_text(&text, "domain")?.unwrap_or_default()
                ));
            }
            let has_metric_ids = regex(r"(?m)^metric_ids:\s*$")?.is_match(&text);
            let has_metrics = regex(r"(?m)^metrics:\s*$")?.is_match(&text);
            if !(has_metric_ids || has_metrics) {
                errors.push(format!(
                    "{}: must define either metric_ids: or metrics:",
                    metrics_file.display()
                ));
            }
            if has_metric_ids && !metric_item_re.is_match(&text) {
                errors.push(format!(
                    "{}: metric_ids must contain at least one snake_case metric id",
                    metrics_file.display()
                ));
            }
            if has_metrics && !metric_entry_re.is_match(&text) {
                errors.push(format!(
                    "{}: metrics entries must include id fields",
                    metrics_file.display()
                ));
            }
        } else {
            errors.push(format!("{}: missing metrics.yaml", dom_dir.display()));
        }

        let artifacts_file = dom_dir.join("artifacts.yaml");
        let artifacts_schema = dom_dir.join("artifacts/_schema.yaml");
        if !artifacts_schema.is_file() {
            errors.push(format!(
                "{}: missing artifacts schema {}",
                dom_dir.display(),
                workspace.rel(&artifacts_schema).display()
            ));
        }
        if artifacts_file.is_file() {
            let text = read_utf8(&artifacts_file)?;
            let keys = top_level_keys(&text)?;
            if artifacts_schema.is_file() {
                let required = required_fields(&artifacts_schema)?;
                let missing = required
                    .iter()
                    .filter(|field| !keys.contains(field.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();
                if !missing.is_empty() {
                    errors.push(format!(
                        "{}: missing required fields from schema: {:?}",
                        artifacts_file.display(),
                        missing
                    ));
                }
                let payload_keys = allowed_payload_keys(&artifacts_schema)?;
                if !payload_keys.is_empty() && !payload_keys.iter().any(|key| keys.contains(key)) {
                    errors.push(format!(
                        "{}: must define at least one payload key from {:?}",
                        artifacts_file.display(),
                        payload_keys
                    ));
                }
            }
            let schema_version = scalar_from_text(&text, "schema_version")?;
            if !schema_version
                .as_deref()
                .is_some_and(|value| value.starts_with("bijux."))
            {
                errors.push(format!(
                    "{}: schema_version must exist and start with 'bijux.'",
                    artifacts_file.display()
                ));
            }
            if scalar_from_text(&text, "domain")?.as_deref() != Some(dom.as_str()) {
                errors.push(format!(
                    "{}: domain must be '{dom}' (got {})",
                    artifacts_file.display(),
                    scalar_from_text(&text, "domain")?.unwrap_or_default()
                ));
            }
            let has_artifact_ids = regex(r"(?m)^artifact_ids:\s*$")?.is_match(&text);
            let has_artifacts = regex(r"(?m)^artifacts:\s*$")?.is_match(&text);
            if !(has_artifact_ids || has_artifacts) {
                errors.push(format!(
                    "{}: must define either artifact_ids: or artifacts:",
                    artifacts_file.display()
                ));
            }
            if has_artifact_ids && !metric_item_re.is_match(&text) {
                errors.push(format!(
                    "{}: artifact_ids must contain at least one snake_case artifact id",
                    artifacts_file.display()
                ));
            }
            if has_artifacts && !metric_entry_re.is_match(&text) {
                errors.push(format!(
                    "{}: artifacts entries must include id fields",
                    artifacts_file.display()
                ));
            }
        } else {
            errors.push(format!("{}: missing artifacts.yaml", dom_dir.display()));
        }

        let mut fixture_pairs = BTreeSet::new();
        for fixture in WalkDir::new(dom_dir.join("fixtures"))
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            if fixture.path().extension().and_then(|ext| ext.to_str()) != Some("txt") {
                continue;
            }
            let stage_id = fixture
                .path()
                .parent()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow!("invalid fixture path {}", fixture.path().display()))?
                .to_string();
            let tool_id = fixture
                .path()
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow!("invalid fixture path {}", fixture.path().display()))?
                .to_string();
            fixture_pairs.insert((stage_id, tool_id));
        }
        for (stage_id, tool_id) in &production_bindings {
            if !stage_id.starts_with(&format!("{dom}.")) {
                continue;
            }
            if !fixture_pairs.contains(&(stage_id.clone(), tool_id.clone())) {
                errors.push(format!(
                    "domain/{dom}/fixtures: missing production fixture for binding ({stage_id}, {tool_id})"
                ));
            }
        }
    }

    if errors.is_empty() {
        return success_line("domain schema: OK");
    }
    failure_block("domain schema check failed", errors)
}

fn check_domain_tool_metadata(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        for tool_path in yaml_files(&dom_dir.join("tools"))? {
            if tool_path.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&tool_path)?;
            if scalar_from_text(&text, "status")?.as_deref() == Some("out_of_scope") {
                continue;
            }
            let tool_id = scalar_from_text(&text, "tool_id")?;
            let citation = scalar_from_text(&text, "citation")?;
            let homepage = scalar_from_text(&text, "homepage")?
                .or_else(|| scalar_from_text(&text, "upstream").ok().flatten());
            let license_id = scalar_from_text(&text, "license-id")?
                .or_else(|| scalar_from_text(&text, "license").ok().flatten());

            if tool_id.is_none() {
                errors.push(format!(
                    "{} missing tool_id",
                    workspace.rel(&tool_path).display()
                ));
            }
            if homepage.is_none() {
                errors.push(format!(
                    "{} missing homepage/upstream",
                    workspace.rel(&tool_path).display()
                ));
            }
            if citation.is_none() {
                errors.push(format!(
                    "{} missing citation",
                    workspace.rel(&tool_path).display()
                ));
            }
            if license_id.is_none() {
                errors.push(format!(
                    "{} missing license-id/license",
                    workspace.rel(&tool_path).display()
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("domain tool metadata: OK");
    }
    failure_block("domain tool metadata check failed", errors)
}

fn external_tools(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let config = load_toml(&workspace.path("configs/domain/external_tools.toml"))?;
    Ok(config
        .get("non_container_tools")
        .and_then(TomlValue::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default())
}

fn check_external_tool_policy(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let external = external_tools(workspace)?;
    let mut registry_tools = BTreeSet::new();
    for path in tool_registry_files(workspace) {
        if !path.is_file() {
            continue;
        }
        for row in toml_tools(&path)? {
            let Some(table) = row.as_table() else {
                continue;
            };
            if let Some(tool_id) = table
                .get("id")
                .or_else(|| table.get("tool_id"))
                .and_then(TomlValue::as_str)
            {
                registry_tools.insert(tool_id.trim().to_string());
            }
        }
    }

    let required = [
        "gatk",
        "picard",
        "preseq",
        "bamutil",
        "ngsbriggs",
        "dustmasker",
        "seqfu",
        "seqprep",
        "seqpurge",
        "diamond",
        "fastq_scan",
    ];
    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        for fixture in WalkDir::new(dom_dir.join("fixtures"))
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            if fixture.path().extension().and_then(|ext| ext.to_str()) != Some("txt") {
                continue;
            }
            let tool = fixture
                .path()
                .file_stem()
                .and_then(|name| name.to_str())
                .ok_or_else(|| anyhow!("invalid fixture file {}", fixture.path().display()))?
                .to_string();
            if !registry_tools.contains(&tool) && !external.contains(&tool) {
                errors.push(format!(
                    "{}: tool '{tool}' missing from registries and external_tools allowlist",
                    workspace.rel(fixture.path()).display()
                ));
            }
        }
    }
    let missing_required = required
        .iter()
        .filter(|tool| !external.contains(**tool))
        .map(|tool| (*tool).to_string())
        .collect::<Vec<_>>();
    if !missing_required.is_empty() {
        errors.push(format!(
            "configs/domain/external_tools.toml missing required external markers: {missing_required:?}"
        ));
    }
    if errors.is_empty() {
        return success_line("external tool policy: OK");
    }
    failure_block("external tool policy check failed", errors)
}

fn check_fixture_contracts(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let external = external_tools(workspace)?;
    let mut errors = Vec::new();
    let stage_re = regex(r#"(?m)^stage_id:\s*"?([^"\n#]+)"?\s*$"#)?;
    let tool_re = regex(r#"(?m)^tool_id:\s*"?([^"\n#]+)"?\s*$"#)?;
    let readme_stage_re = regex(r"(?im)^\s*[-*]\s*`?([^`\n]+)`?\s*:.*intent")?;
    let snake_case = regex(r"^[a-z0-9_]+$")?;

    for dom_dir in domain_directories(workspace)? {
        let fixtures_root = dom_dir.join("fixtures");
        let readme = fixtures_root.join("README.md");
        let readme_text = if readme.is_file() {
            read_utf8(&readme)?
        } else {
            errors.push(format!("{} missing", workspace.rel(&readme).display()));
            String::new()
        };

        let mut known_stage_ids = BTreeSet::new();
        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&stage_file)?;
            if let Some(captures) = stage_re.captures(&text) {
                if let Some(stage_id) = captures.get(1) {
                    known_stage_ids.insert(stage_id.as_str().trim().to_string());
                }
            }
        }

        let mut known_tools = BTreeSet::new();
        for tool_file in yaml_files(&dom_dir.join("tools"))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&tool_file)?;
            let tool_id = tool_re
                .captures(&text)
                .and_then(|captures| captures.get(1))
                .map(|value| value.as_str().trim().to_string())
                .unwrap_or_else(|| {
                    tool_file
                        .file_stem()
                        .and_then(|name| name.to_str())
                        .unwrap_or_default()
                        .to_string()
                });
            known_tools.insert(tool_id);
        }

        if fixtures_root.is_dir() {
            let mut stage_dirs = fs::read_dir(&fixtures_root)
                .with_context(|| format!("read {}", fixtures_root.display()))?
                .filter_map(std::result::Result::ok)
                .filter_map(|entry| match entry.file_type() {
                    Ok(file_type) if file_type.is_dir() => Some(entry.path()),
                    _ => None,
                })
                .collect::<Vec<_>>();
            stage_dirs.sort();

            for stage_dir in stage_dirs {
                let stage_name = stage_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| anyhow!("invalid fixture directory {}", stage_dir.display()))?
                    .to_string();
                if !known_stage_ids.contains(&stage_name) {
                    errors.push(format!(
                        "{}: fixture stage directory is not a known stage_id",
                        workspace.rel(&stage_dir).display()
                    ));
                }
                if !readme_text.is_empty() {
                    if !readme_text.contains(&stage_name) {
                        errors.push(format!(
                            "{}: missing fixture directory listing for '{stage_name}'",
                            workspace.rel(&readme).display()
                        ));
                    }
                    let has_intent = readme_stage_re.captures_iter(&readme_text).any(|captures| {
                        captures
                            .get(1)
                            .is_some_and(|value| value.as_str().trim() == stage_name)
                    });
                    if !has_intent {
                        errors.push(format!(
                            "{}: '{stage_name}' entry must include intent",
                            workspace.rel(&readme).display()
                        ));
                    }
                }

                let mut fixture_files = fs::read_dir(&stage_dir)
                    .with_context(|| format!("read {}", stage_dir.display()))?
                    .filter_map(std::result::Result::ok)
                    .filter(|entry| {
                        entry.path().extension().and_then(|ext| ext.to_str()) == Some("txt")
                    })
                    .collect::<Vec<_>>();
                fixture_files.sort_by_key(std::fs::DirEntry::path);

                for fixture in fixture_files {
                    let path = fixture.path();
                    let text = read_utf8(&path)?.trim().to_string();
                    if !text.contains('=') {
                        let parts = text.split_whitespace().collect::<Vec<_>>();
                        if parts.len() < 2 {
                            errors.push(format!(
                                "{}: invalid fixture format",
                                workspace.rel(&path).display()
                            ));
                            continue;
                        }
                        let tool = parts[1].trim().to_string();
                        if !known_stage_ids.contains(&stage_name) {
                            errors.push(format!(
                                "{}: fixture path stage '{stage_name}' is unknown",
                                workspace.rel(&path).display()
                            ));
                        }
                        if !known_tools.contains(&tool) && !external.contains(&tool) {
                            errors.push(format!(
                                "{}: legacy fixture tool '{tool}' not found in domain tools or external tools",
                                workspace.rel(&path).display()
                            ));
                        }
                        if !external.contains(&tool) {
                            errors.push(format!(
                                "{}: legacy fixture format; use key=value contract fields",
                                workspace.rel(&path).display()
                            ));
                        }
                        continue;
                    }

                    let kv = text
                        .lines()
                        .filter_map(|line| line.split_once('='))
                        .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                        .collect::<BTreeMap<_, _>>();
                    for required_key in [
                        "tool",
                        "tool_version",
                        "command",
                        "args",
                        "expected_outputs",
                        "expected_stdout_patterns",
                        "stage",
                    ] {
                        if !kv.contains_key(required_key) {
                            errors.push(format!(
                                "{}: missing required key '{required_key}'",
                                workspace.rel(&path).display()
                            ));
                        }
                    }
                    if let Some(tool) = kv.get("tool") {
                        if !snake_case.is_match(tool) {
                            errors.push(format!(
                                "{}: tool id '{}' must be snake_case ([a-z0-9_]+)",
                                workspace.rel(&path).display(),
                                tool
                            ));
                        }
                        let stem = path
                            .file_stem()
                            .and_then(|name| name.to_str())
                            .unwrap_or_default();
                        if tool != stem {
                            errors.push(format!(
                                "{}: tool field '{}' must match fixture filename stem '{}'",
                                workspace.rel(&path).display(),
                                tool,
                                stem
                            ));
                        }
                        if !known_tools.contains(tool) && !external.contains(tool) {
                            errors.push(format!(
                                "{}: tool '{}' not found in domain tools or external tools policy",
                                workspace.rel(&path).display(),
                                tool
                            ));
                        }
                        let shipping = kv.get("shipping").cloned().unwrap_or_default();
                        if shipping == "external" && !external.contains(tool) {
                            errors.push(format!(
                                "{}: shipping=external requires tool in configs/domain/external_tools.toml",
                                workspace.rel(&path).display()
                            ));
                        }
                        if external.contains(tool) && shipping != "external" {
                            errors.push(format!(
                                "{}: external tool '{}' must declare shipping=external",
                                workspace.rel(&path).display(),
                                tool
                            ));
                        }
                    }
                    if let Some(stage_value) = kv.get("stage") {
                        if stage_value != &stage_name {
                            errors.push(format!(
                                "{}: stage mismatch ({} != {})",
                                workspace.rel(&path).display(),
                                stage_value,
                                stage_name
                            ));
                        }
                    }
                    if !known_stage_ids.contains(&stage_name) {
                        errors.push(format!(
                            "{}: fixture path stage '{}' is unknown",
                            workspace.rel(&path).display(),
                            stage_name
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        return success_line("fixture contracts: OK");
    }
    failure_block("fixture contracts check failed", errors)
}

#[derive(Debug, Serialize, PartialEq, Eq)]
struct InventoryRow {
    domain: String,
    stages: usize,
    tools: usize,
    fixture_stage_dirs: usize,
    fixture_files: usize,
    has_artifacts_yaml: bool,
    has_metrics_yaml: bool,
    has_default_settings_doc: bool,
}

fn build_inventory_rows(workspace: &Workspace) -> Result<Vec<InventoryRow>> {
    let mut rows = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        let domain = dom_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid domain directory {}", dom_dir.display()))?
            .to_string();
        let stages = yaml_files(&dom_dir.join("stages"))?
            .into_iter()
            .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("_schema.yaml"))
            .count();
        let tools = yaml_files(&dom_dir.join("tools"))?
            .into_iter()
            .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("_schema.yaml"))
            .count();
        let fixtures_root = dom_dir.join("fixtures");
        let fixture_stage_dirs = if fixtures_root.is_dir() {
            fs::read_dir(&fixtures_root)
                .with_context(|| format!("read {}", fixtures_root.display()))?
                .filter_map(std::result::Result::ok)
                .filter_map(|entry| match entry.file_type() {
                    Ok(file_type) if file_type.is_dir() => Some(()),
                    _ => None,
                })
                .count()
        } else {
            0
        };
        let fixture_files = if fixtures_root.is_dir() {
            WalkDir::new(&fixtures_root)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .filter(|entry| {
                    entry.file_type().is_file()
                        && entry.path().extension().and_then(|ext| ext.to_str()) == Some("txt")
                })
                .count()
        } else {
            0
        };
        rows.push(InventoryRow {
            domain,
            stages,
            tools,
            fixture_stage_dirs,
            fixture_files,
            has_artifacts_yaml: dom_dir.join("artifacts.yaml").is_file(),
            has_metrics_yaml: dom_dir.join("metrics.yaml").is_file(),
            has_default_settings_doc: dom_dir.join("docs/DEFAULT_SETTINGS.md").is_file(),
        });
    }
    Ok(rows)
}

fn render_inventory_json(rows: &[InventoryRow]) -> Result<String> {
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&serde_json::json!({
            "schema_version": "bijux.domain.inventory.v1",
            "domains": rows,
        }))?
    ))
}

fn render_inventory_markdown(rows: &[InventoryRow]) -> String {
    let mut lines = vec![
        "# Domain Inventory".to_string(),
        String::new(),
        "| Domain | Stages | Tools | Fixture Stage Dirs | Fixture Files | artifacts.yaml | metrics.yaml | DEFAULT_SETTINGS.md |".to_string(),
        "|---|---:|---:|---:|---:|:---:|:---:|:---:|".to_string(),
    ];
    for row in rows {
        lines.push(format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} |",
            row.domain,
            row.stages,
            row.tools,
            row.fixture_stage_dirs,
            row.fixture_files,
            yes_no(row.has_artifacts_yaml),
            yes_no(row.has_metrics_yaml),
            yes_no(row.has_default_settings_doc)
        ));
    }
    format!("{}\n", lines.join("\n"))
}

fn yes_no(value: bool) -> &'static str {
    if value {
        "yes"
    } else {
        "no"
    }
}

fn check_inventory(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let rows_first = build_inventory_rows(workspace)?;
    let json_first = render_inventory_json(&rows_first)?;
    let md_first = render_inventory_markdown(&rows_first);
    let rows_second = build_inventory_rows(workspace)?;
    let json_second = render_inventory_json(&rows_second)?;
    let md_second = render_inventory_markdown(&rows_second);

    if json_first != json_second {
        return Ok(DomainCommandOutcome::failure(
            "domain inventory is non-deterministic across consecutive generations\n",
        ));
    }
    if md_first != md_second {
        return Ok(DomainCommandOutcome::failure(
            "domain inventory markdown is non-deterministic across consecutive generations\n",
        ));
    }

    let out_json = workspace.path("artifacts/domain/inventory.json");
    let out_md = workspace.path("artifacts/domain/inventory.md");
    write_utf8(&out_json, &json_first)?;
    write_utf8(&out_md, &md_first)?;
    success_line(format!(
        "domain inventory: OK ({}, {})",
        out_json.display(),
        out_md.display()
    ))
}

fn check_orphan_files(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let external_tools = external_tools(workspace)?;
    let mut registry_tools_by_domain = BTreeMap::<String, BTreeSet<String>>::new();
    let registry_dir = workspace.path("configs/ci/registry");
    let mut registry_files = fs::read_dir(&registry_dir)
        .with_context(|| format!("read {}", registry_dir.display()))?
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with("tool_registry") && name.ends_with(".toml"))
        })
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    registry_files.sort();
    for registry in registry_files {
        for row in toml_tools(&registry)? {
            let Some(table) = row.as_table() else {
                continue;
            };
            let tool_id = table
                .get("tool_id")
                .or_else(|| table.get("id"))
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            let status = table
                .get("status")
                .and_then(TomlValue::as_str)
                .unwrap_or_default()
                .trim()
                .to_string();
            if tool_id.is_empty()
                || tool_id.contains('.')
                || !matches!(status.as_str(), "production" | "supported")
            {
                continue;
            }
            for binding in table
                .get("bindings")
                .and_then(TomlValue::as_array)
                .into_iter()
                .flatten()
            {
                let Some(stage_id) = binding.as_str() else {
                    continue;
                };
                let Some((domain, _)) = stage_id.split_once('.') else {
                    continue;
                };
                registry_tools_by_domain
                    .entry(domain.to_string())
                    .or_default()
                    .insert(tool_id.clone());
            }
        }
    }

    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        let dom = dom_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("invalid domain directory {}", dom_dir.display()))?
            .to_string();
        let index = dom_dir.join("index.yaml");
        if !index.is_file() {
            continue;
        }
        let text = read_utf8(&index)?;
        let indexed_stages = list_block(&text, "stage_ids")?
            .into_iter()
            .collect::<BTreeSet<_>>();
        let indexed_tools = list_block(&text, "tool_ids")?
            .into_iter()
            .collect::<BTreeSet<_>>();
        let mut fixture_tools = BTreeSet::new();
        for fixture in WalkDir::new(dom_dir.join("fixtures"))
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| {
                entry.file_type().is_file()
                    && entry.path().extension().and_then(|ext| ext.to_str()) == Some("txt")
            })
        {
            if let Some(stem) = fixture.path().file_stem().and_then(|name| name.to_str()) {
                fixture_tools.insert(stem.to_string());
            }
        }

        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let stage_id = declared_stage_key(&stage_file)?.unwrap_or_else(|| {
                stage_file
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .to_string()
            });
            if !indexed_stages.contains(&stage_id) {
                errors.push(format!(
                    "{}: orphan stage file not referenced by index.yaml",
                    workspace.rel(&stage_file).display()
                ));
            }
        }

        let mut domain_tool_ids = BTreeSet::new();
        for tool_file in yaml_files(&dom_dir.join("tools"))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let tool_id = declared_tool_key(&tool_file)?.unwrap_or_else(|| {
                tool_file
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .unwrap_or_default()
                    .to_string()
            });
            domain_tool_ids.insert(tool_id.clone());
            if !indexed_tools.contains(&tool_id)
                && !fixture_tools.contains(&tool_id)
                && !registry_tools_by_domain
                    .get(&dom)
                    .is_some_and(|tools| tools.contains(&tool_id))
            {
                errors.push(format!(
                    "{}: orphan tool file not referenced by index.yaml, fixtures, or registry bindings",
                    workspace.rel(&tool_file).display()
                ));
            }
        }

        for registry_tool in registry_tools_by_domain
            .get(&dom)
            .cloned()
            .unwrap_or_default()
        {
            if !domain_tool_ids.contains(&registry_tool) && !external_tools.contains(&registry_tool)
            {
                errors.push(format!(
                    "domain/{dom}/tools: missing tool yaml for registry-bound tool '{registry_tool}' (or declare external tool policy)"
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("orphan stage/tool: OK");
    }
    failure_block("orphan stage/tool check failed", errors)
}

fn planner_stage_ids(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let mut stage_ids = BTreeSet::new();
    for rel in [
        "configs/ci/stages/stages.toml",
        "configs/ci/stages/stages_vcf.toml",
        "configs/ci/stages/stages_vcf_downstream.toml",
    ] {
        for row in toml_stages(&workspace.path(rel))? {
            if let Some(stage_id) = row
                .as_table()
                .and_then(|table| table.get("id"))
                .and_then(TomlValue::as_str)
            {
                stage_ids.insert(stage_id.trim().to_string());
            }
        }
    }
    Ok(stage_ids)
}

fn check_planner_fixture_coverage(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut errors = Vec::new();
    for stage_id in planner_stage_ids(workspace)? {
        let domain = stage_id.split('.').next().unwrap_or_default();
        let fixture_dir = workspace.path(&format!("domain/{domain}/fixtures/{stage_id}"));
        let has_files = fixture_dir.is_dir()
            && WalkDir::new(&fixture_dir)
                .into_iter()
                .filter_map(std::result::Result::ok)
                .any(|entry| entry.file_type().is_file());
        if !has_files {
            errors.push(format!(
                "{} missing fixture files for planner stage '{stage_id}'",
                workspace.rel(&fixture_dir).display()
            ));
        }
    }
    if errors.is_empty() {
        return success_line("planner fixture coverage: OK");
    }
    failure_block("planner fixture coverage check failed", errors)
}

fn check_planner_stage_coverage(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let planner_stage_ids = planner_stage_ids(workspace)?;
    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        for stage_file in yaml_files(&dom_dir.join("stages"))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&stage_file)?;
            let Some(stage_id) = scalar_from_text(&text, "stage_id")? else {
                continue;
            };
            let status = scalar_from_text(&text, "status")?.unwrap_or_default();
            if status != "supported" {
                continue;
            }
            if !planner_stage_ids.contains(&stage_id) {
                errors.push(format!(
                    "{}: supported stage '{stage_id}' missing planner coverage in configs/ci/stages/*.toml",
                    workspace.rel(&stage_file).display()
                ));
            }
        }
    }
    if errors.is_empty() {
        return success_line("planner stage coverage: OK");
    }
    failure_block("planner stage coverage check failed", errors)
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn check_reference_bundle_lock(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let catalog = workspace.path("configs/runtime/reference_bundles.toml");
    let lock = workspace.path("configs/runtime/reference_bundles_lock.sha256");
    let materialization_lock_json = workspace.path("configs/runtime/references/locks/lock.json");
    let materialization_lock_sha =
        workspace.path("configs/runtime/references/locks/lock.json.sha256");

    if !catalog.is_file() {
        return Ok(DomainCommandOutcome::failure(format!(
            "reference bundle lock check: missing {}\n",
            catalog.display()
        )));
    }
    if !lock.is_file() {
        return Ok(DomainCommandOutcome::failure(format!(
            "reference bundle lock check: missing {}\n",
            lock.display()
        )));
    }
    let expected =
        sha256_hex(&fs::read(&catalog).with_context(|| format!("read {}", catalog.display()))?);
    let actual = read_utf8(&lock)?.trim().to_string();
    if expected != actual {
        return Ok(DomainCommandOutcome::failure(format!(
            "reference bundle lock drift: {} is stale; update it after bundle changes\nexpected={expected}\nactual={actual}\n",
            lock.display()
        )));
    }

    let mut stdout = String::from("reference bundle lock: OK\n");
    if materialization_lock_json.is_file() || materialization_lock_sha.is_file() {
        if !materialization_lock_json.is_file() {
            return Ok(DomainCommandOutcome::failure(format!(
                "reference materialization lock check: missing {}\n",
                materialization_lock_json.display()
            )));
        }
        if !materialization_lock_sha.is_file() {
            return Ok(DomainCommandOutcome::failure(format!(
                "reference materialization lock check: missing {}\n",
                materialization_lock_sha.display()
            )));
        }
        let expected = sha256_hex(
            &fs::read(&materialization_lock_json)
                .with_context(|| format!("read {}", materialization_lock_json.display()))?,
        );
        let actual = read_utf8(&materialization_lock_sha)?
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .trim()
            .to_string();
        if expected != actual {
            return Ok(DomainCommandOutcome::failure(format!(
                "reference materialization lock drift: {} is stale\nexpected={expected}\nactual={actual}\n",
                materialization_lock_sha.display()
            )));
        }
        stdout.push_str("reference materialization lock: OK\n");
    }
    Ok(DomainCommandOutcome::success(stdout))
}

fn parse_stage_catalog(path: &Path, const_name: &str) -> Result<BTreeSet<String>> {
    let text = read_utf8(path)?;
    let pattern = format!(
        r"(?s)pub\s+const\s+{}:\s*&\[\s*&str\s*\]\s*=\s*&\[(.*?)\];",
        regex::escape(const_name)
    );
    let captures = regex(&pattern)?
        .captures(&text)
        .ok_or_else(|| anyhow!("missing {const_name} in {}", path.display()))?;
    let body = captures
        .get(1)
        .map(|value| value.as_str())
        .ok_or_else(|| anyhow!("missing catalog body for {const_name}"))?;
    let item_re = regex(r#""([a-z0-9_.]+)""#)?;
    Ok(item_re
        .captures_iter(body)
        .filter_map(|captures| captures.get(1))
        .map(|value| value.as_str().to_string())
        .collect())
}

fn check_rust_stage_catalog_parity(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let specs = [
        (
            "fastq",
            workspace.path("crates/bijux-dna-domain-fastq/src/id_catalog.rs"),
            "FASTQ_STAGE_ID_CATALOG",
        ),
        (
            "bam",
            workspace.path("crates/bijux-dna-domain-bam/src/types/mod.rs"),
            "BAM_STAGE_ID_CATALOG",
        ),
        (
            "vcf",
            workspace.path("crates/bijux-dna-domain-vcf/src/lib.rs"),
            "VCF_STAGE_ID_CATALOG",
        ),
    ];

    let mut errors = Vec::new();
    for (domain, path, const_name) in specs {
        let domain_ids = list_block(
            &read_utf8(&workspace.path(&format!("domain/{domain}/index.yaml")))?,
            "stage_ids",
        )?
        .into_iter()
        .collect::<BTreeSet<_>>();
        let rust_ids = parse_stage_catalog(&path, const_name)?;
        for missing in domain_ids.difference(&rust_ids) {
            errors.push(format!(
                "{}: {const_name} missing domain stage '{}'",
                workspace.rel(&path).display(),
                missing
            ));
        }
        for extra in rust_ids.difference(&domain_ids) {
            errors.push(format!(
                "{}: {const_name} has stale non-domain stage '{}'",
                workspace.rel(&path).display(),
                extra
            ));
        }
    }
    if errors.is_empty() {
        return success_line("rust stage catalog parity: OK");
    }
    failure_block("rust stage catalog parity check failed", errors)
}

fn check_shared_tools(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let config = load_toml(&workspace.path("configs/domain/shared_tools.toml"))?;
    let shared = config
        .get("shared_tools")
        .and_then(TomlValue::as_table)
        .cloned()
        .unwrap_or_default();
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
            row.insert(
                "path".to_string(),
                workspace.rel(&tool_file).display().to_string(),
            );
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
        let mut domains_actual = rows
            .iter()
            .filter_map(|row| row.get("domain").cloned())
            .collect::<Vec<_>>();
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

fn check_ssot_authority(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let doc = workspace.path("docs/10-architecture/SSOT.md");
    let doc_text = read_utf8(&doc)?;
    if !doc_text.contains("domain/*/**/*.yaml") || !doc_text.contains("source of truth") {
        return Ok(DomainCommandOutcome::failure(
            "ssot authority check: docs/10-architecture/SSOT.md must declare domain/*/**/*.yaml as source of truth\n",
        ));
    }

    let mut errors = Vec::new();
    for dom_dir in domain_directories(workspace)? {
        let index_path = dom_dir.join("index.yaml");
        if !index_path.is_file() {
            continue;
        }
        let text = read_utf8(&index_path)?;
        let Some(version) = scalar_from_text(&text, "domain_version")? else {
            errors.push(format!(
                "{} missing domain_version: v1|v2",
                workspace.rel(&index_path).display()
            ));
            continue;
        };
        if !matches!(version.as_str(), "v1" | "v2") {
            errors.push(format!(
                "{} has invalid domain_version '{}' (expected v1|v2)",
                workspace.rel(&index_path).display(),
                version
            ));
        }
        if dom_dir.file_name().and_then(|name| name.to_str()) == Some("vcf") && version != "v2" {
            errors.push("domain/vcf/index.yaml must declare domain_version: v2".to_string());
        }
    }
    if errors.is_empty() {
        return success_line("ssot authority/version: OK");
    }
    failure_block("ssot authority check failed", errors)
}

fn check_tool_container_parity(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let external = external_tools(workspace)?;
    let docker_tools = fs::read_dir(workspace.path("containers/docker/arm64"))
        .with_context(|| {
            format!(
                "read {}",
                workspace.path("containers/docker/arm64").display()
            )
        })?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name();
            let name = name.to_string_lossy();
            name.strip_prefix("Dockerfile.").map(ToString::to_string)
        })
        .collect::<BTreeSet<_>>();
    let apptainer_tools = fs::read_dir(workspace.path("containers/apptainer/shared"))
        .with_context(|| {
            format!(
                "read {}",
                workspace.path("containers/apptainer/shared").display()
            )
        })?
        .filter_map(std::result::Result::ok)
        .filter_map(|entry| {
            if entry.path().extension().and_then(|ext| ext.to_str()) == Some("def") {
                entry
                    .path()
                    .file_stem()
                    .and_then(|name| name.to_str())
                    .map(ToString::to_string)
            } else {
                None
            }
        })
        .collect::<BTreeSet<_>>();
    let all_container_tools = docker_tools
        .into_iter()
        .chain(apptainer_tools)
        .collect::<BTreeSet<_>>();

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
                    if candidates
                        .iter()
                        .all(|candidate| !all_container_tools.contains(candidate))
                    {
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
                if candidates
                    .iter()
                    .all(|candidate| !all_container_tools.contains(candidate))
                {
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

fn generate_index(workspace: &Workspace, args: &[String]) -> Result<DomainCommandOutcome> {
    if args.len() != 1 {
        return Ok(DomainCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr:
                "Usage: cargo run -p bijux-dna-dev -- domain run generate-index -- <domain>|--all\n"
                    .to_string(),
        });
    }
    let domains = if args[0] == "--all" {
        domain_directories(workspace)?
            .into_iter()
            .filter_map(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map(ToString::to_string)
            })
            .collect::<Vec<_>>()
    } else {
        vec![args[0].clone()]
    };
    let mut stdout = String::new();
    for dom in domains {
        let index_path = workspace.path(&format!("domain/{dom}/index.yaml"));
        let rendered = render_domain_index(workspace, &dom)?;
        write_utf8(&index_path, &rendered)?;
        stdout.push_str("generated ");
        stdout.push_str(&index_path.display().to_string());
        stdout.push('\n');
    }
    Ok(DomainCommandOutcome::success(stdout))
}

fn generate_inventory(workspace: &Workspace, args: &[String]) -> Result<DomainCommandOutcome> {
    if args.len() > 2 {
        return Ok(DomainCommandOutcome {
            exit_code: 2,
            stdout: String::new(),
            stderr: "Usage: cargo run -p bijux-dna-dev -- domain run generate-inventory -- [<json-path> [<markdown-path>]]\n".to_string(),
        });
    }
    let out_json = args.first().map_or_else(
        || workspace.path("artifacts/domain/inventory.json"),
        PathBuf::from,
    );
    let out_md = args.get(1).map_or_else(
        || workspace.path("artifacts/domain/inventory.md"),
        PathBuf::from,
    );
    let rows = build_inventory_rows(workspace)?;
    write_utf8(&out_json, &render_inventory_json(&rows)?)?;
    write_utf8(&out_md, &render_inventory_markdown(&rows))?;
    Ok(DomainCommandOutcome::success(format!(
        "generated {}\ngenerated {}\n",
        out_json.display(),
        out_md.display()
    )))
}

fn inventory_drift(workspace: &Workspace) -> Result<DomainCommandOutcome> {
    let mut domain_tools = BTreeSet::new();
    for domain in ["fastq", "bam"] {
        for tool_file in yaml_files(&workspace.path(&format!("domain/{domain}/tools")))? {
            if tool_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&tool_file)?;
            let status = scalar_from_text(&text, "status")?.unwrap_or_default();
            if matches!(status.as_str(), "production" | "supported") {
                if let Some(tool_id) = scalar_from_text(&text, "tool_id")? {
                    domain_tools.insert(tool_id);
                }
            }
        }
    }

    let mut domain_stages = BTreeSet::new();
    for domain in ["fastq", "bam"] {
        for stage_file in yaml_files(&workspace.path(&format!("domain/{domain}/stages")))? {
            if stage_file.file_name().and_then(|name| name.to_str()) == Some("_schema.yaml") {
                continue;
            }
            let text = read_utf8(&stage_file)?;
            let status = scalar_from_text(&text, "status")?.unwrap_or_default();
            if matches!(status.as_str(), "production" | "supported") {
                if let Some(stage_id) = scalar_from_text(&text, "stage_id")? {
                    domain_stages.insert(stage_id);
                }
            }
        }
    }

    let registry_tools = cargo_registry_list_tools(workspace)?;
    let registry_stages = cargo_registry_list_stages(workspace)?;

    let tool_ref_re = regex(r#"ToolId::from_static\("([a-z0-9_\-]+)"\)"#)?;
    let stage_ref_re = regex(r#"StageId::from_static\("([a-z0-9._-]+)"\)"#)?;
    let mut code_tools = BTreeSet::new();
    let mut code_stages_raw = BTreeSet::new();
    let synthetic_core_test = format!("{}test", id_catalog::CORE_PREFIX);
    for entry in WalkDir::new(workspace.path("crates"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let text = read_utf8(entry.path()).unwrap_or_default();
        for captures in tool_ref_re.captures_iter(&text) {
            if let Some(value) = captures.get(1).map(|capture| capture.as_str()) {
                if !matches!(value, "tool" | "planner" | "unknown") {
                    code_tools.insert(value.to_string());
                }
            }
        }
        for captures in stage_ref_re.captures_iter(&text) {
            if let Some(value) = captures.get(1).map(|capture| capture.as_str()) {
                if value == synthetic_core_test
                    || value == "report.aggregate"
                    || value.starts_with("stage.")
                    || value == id_catalog::FASTQ_PREPROCESS
                {
                    continue;
                }
                code_stages_raw.insert(value.to_string());
            }
        }
    }
    let code_stages = code_stages_raw
        .into_iter()
        .filter(|stage_id| domain_stages.contains(stage_id))
        .collect::<BTreeSet<_>>();

    let stage_tools_re = regex(r"stage-tools ([a-z0-9._-]+) all")?;
    let mut make_stage_ids = BTreeSet::new();
    for file in WalkDir::new(workspace.path("makes"))
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let text = read_utf8(file.path()).unwrap_or_default();
        for captures in stage_tools_re.captures_iter(&text) {
            if let Some(stage_id) = captures.get(1).map(|value| value.as_str()) {
                make_stage_ids.insert(stage_id.to_string());
            }
        }
    }
    let makefile_text = read_utf8(&workspace.path("Makefile")).unwrap_or_default();
    for captures in stage_tools_re.captures_iter(&makefile_text) {
        if let Some(stage_id) = captures.get(1).map(|value| value.as_str()) {
            make_stage_ids.insert(stage_id.to_string());
        }
    }
    let mut make_tools = BTreeSet::new();
    for stage_id in make_stage_ids {
        make_tools.extend(cargo_registry_stage_tools(workspace, &stage_id)?);
    }

    let mut diffs = Vec::new();
    push_diff(
        &mut diffs,
        &domain_tools,
        &registry_tools,
        "domain tools missing from registry",
    );
    push_diff(
        &mut diffs,
        &code_tools,
        &registry_tools,
        "code-referenced tools missing from registry",
    );
    push_diff(
        &mut diffs,
        &make_tools,
        &registry_tools,
        "make-referenced tools missing from registry",
    );
    push_diff(
        &mut diffs,
        &registry_tools,
        &domain_tools,
        "registry tools missing from domain",
    );
    push_diff(
        &mut diffs,
        &domain_stages,
        &registry_stages,
        "domain stages missing from generated configs/ci/stages/stages.toml",
    );
    push_diff(
        &mut diffs,
        &registry_stages,
        &domain_stages,
        "generated configs/ci/stages/stages.toml stages missing from domain",
    );
    push_diff(
        &mut diffs,
        &code_stages,
        &registry_stages,
        "code-referenced stages missing from generated configs/ci/stages/stages.toml",
    );

    let mut stdout = String::new();
    if !diffs.is_empty() {
        stdout.push_str(&diffs.join(""));
    }
    stdout.push_str("--- inventory counts ---\n");
    stdout.push_str(&format!("domain:   {}\n", domain_tools.len()));
    stdout.push_str(&format!("registry: {}\n", registry_tools.len()));
    stdout.push_str(&format!("code:     {}\n", code_tools.len()));
    stdout.push_str(&format!("make:     {}\n", make_tools.len()));
    stdout.push_str(&format!("stages(domain): {}\n", domain_stages.len()));
    stdout.push_str(&format!("stages(registry): {}\n", registry_stages.len()));
    stdout.push_str(&format!("stages(code): {}\n", code_stages.len()));

    if diffs.is_empty() {
        stdout.push_str("domain-inventory-drift: OK\n");
        return Ok(DomainCommandOutcome::success(stdout));
    }

    Ok(DomainCommandOutcome {
        exit_code: 1,
        stdout,
        stderr: "domain-inventory-drift: mismatch detected\n".to_string(),
    })
}

fn push_diff(
    output: &mut Vec<String>,
    left: &BTreeSet<String>,
    right: &BTreeSet<String>,
    title: &str,
) {
    let missing = left.difference(right).cloned().collect::<Vec<_>>();
    if missing.is_empty() {
        return;
    }
    let mut block = String::new();
    block.push_str("[DIFF] ");
    block.push_str(title);
    block.push('\n');
    for entry in missing {
        block.push_str("  - ");
        block.push_str(&entry);
        block.push('\n');
    }
    output.push(block);
}

fn lock_registry(workspace: &Workspace, args: &[String]) -> Result<DomainCommandOutcome> {
    let print_only = match args {
        [] => false,
        [single] if single == "--print" => true,
        [single] if single == "--help" || single == "-h" => {
            return Ok(DomainCommandOutcome::success(
                "Usage: cargo run -p bijux-dna-dev -- domain run lock-registry -- [--print]\n",
            ));
        }
        _ => {
            return Ok(DomainCommandOutcome {
                exit_code: 2,
                stdout: String::new(),
                stderr: "unknown arg\n".to_string(),
            });
        }
    };

    let lock_doc = workspace.path("configs/ci/registry/LOCK_RULES.md");
    if !lock_doc.is_file() {
        bail!("missing {}", lock_doc.display());
    }
    let inputs = [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
        "configs/ci/registry/domains.toml",
        "configs/ci/registry/deprecations.toml",
    ];
    let mut payload = String::new();
    for rel in inputs {
        let path = workspace.path(rel);
        let sha = sha256_hex(&fs::read(&path).with_context(|| format!("read {}", path.display()))?);
        payload.push_str(rel);
        payload.push(' ');
        payload.push_str(&sha);
        payload.push('\n');
    }
    let lock_sha = sha256_hex(payload.as_bytes());
    if print_only {
        return Ok(DomainCommandOutcome::success(format!("{lock_sha}\n")));
    }

    let lock_file = workspace.path("configs/ci/registry/tool_registry_lock.sha256");
    let marker_file = workspace.path("artifacts/configs/tool_registry_lock.marker");
    write_utf8(&lock_file, &format!("{lock_sha}\n"))?;
    write_utf8(
        &marker_file,
        &format!("{REGISTRY_LOCK_GENERATED_BY}\nlock_sha256={lock_sha}\n"),
    )?;
    success_line(format!(
        "updated {} (rules: configs/ci/registry/LOCK_RULES.md)",
        lock_file.display()
    ))
}

fn validate(workspace: &Workspace, args: &[String]) -> Result<DomainCommandOutcome> {
    let allow_non_artifacts = match args {
        [] => false,
        [single] if single == "--allow-non-artifacts" => true,
        [single] if single == "--help" || single == "-h" => {
            return Ok(DomainCommandOutcome::success(
                "Usage: cargo run -p bijux-dna-dev -- domain run validate -- [--allow-non-artifacts]\n",
            ));
        }
        _ => {
            return Ok(DomainCommandOutcome {
                exit_code: 2,
                stdout: String::new(),
                stderr: "Usage: cargo run -p bijux-dna-dev -- domain run validate -- [--allow-non-artifacts]\n".to_string(),
            });
        }
    };

    let checks = [
        check_domain_layout(workspace)?,
        check_domain_schema(workspace)?,
        check_domain_index(workspace)?,
        check_ssot_authority(workspace)?,
        check_rust_stage_catalog_parity(workspace)?,
        check_shared_tools(workspace)?,
        check_tool_container_parity(workspace)?,
        check_domain_tool_metadata(workspace)?,
        check_planner_stage_coverage(workspace)?,
        check_planner_fixture_coverage(workspace)?,
        check_default_settings_docs(workspace)?,
        check_fixture_contracts(workspace)?,
        check_orphan_files(workspace)?,
        check_doc_links(workspace)?,
        check_external_tool_policy(workspace)?,
        check_reference_bundle_lock(workspace)?,
        check_inventory(workspace)?,
    ];
    let mut stdout = String::new();
    let mut stderr = String::new();
    for outcome in checks {
        stdout.push_str(&outcome.stdout);
        stderr.push_str(&outcome.stderr);
        if !outcome.is_success() {
            return Ok(DomainCommandOutcome {
                exit_code: outcome.exit_code,
                stdout,
                stderr,
            });
        }
    }

    let env = if allow_non_artifacts {
        vec![
            ("TZ".to_string(), "UTC".to_string()),
            ("LC_ALL".to_string(), "C".to_string()),
        ]
    } else {
        artifact_env(workspace)?
    };
    let compiler = command_runner(workspace).run_owned_with_env(
        "cargo",
        &[
            "run".to_string(),
            "-p".to_string(),
            "bijux-dna-domain-compiler".to_string(),
            "--bin".to_string(),
            "domain_validate".to_string(),
            "--".to_string(),
            "--domain-dir".to_string(),
            workspace.path("domain").display().to_string(),
        ],
        &env,
    )?;
    let compiler_outcome = DomainCommandOutcome::from_output(compiler);
    stdout.push_str(&compiler_outcome.stdout);
    stderr.push_str(&compiler_outcome.stderr);
    Ok(DomainCommandOutcome {
        exit_code: compiler_outcome.exit_code,
        stdout,
        stderr,
    })
}
