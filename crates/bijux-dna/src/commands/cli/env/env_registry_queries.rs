use std::collections::{BTreeSet, HashMap, HashSet};
use std::io::Read;
use std::path::Path;
use std::time::{Duration, SystemTime};

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::env::{
    available_runners, cache_dir, docker_image_exists, resolve_image, run_shell_capture,
    run_smoke_script, run_smoke_script_batch, PlatformSpec, RuntimeKind, ToolImageSpec,
};
use regex::Regex;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// # Errors
/// Returns an error if image resolution fails.
pub fn print_env_images<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) -> Result<()> {
    let mut entries: Vec<_> = catalog.iter().collect();
    entries.sort_by_key(|(name, _)| *name);
    for (name, spec) in entries {
        let resolved = resolve_image(spec, platform)?;
        let digest = spec.digest.as_deref().unwrap_or("no digest");
        println!("{name}: {} ({digest})", resolved.full_name);
    }
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read.
pub fn print_env_registry_list(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    println!("tool\thas_docker\thas_apptainer\thas_smoke\tpinned");
    for row in parse_tools_registry_rows(&raw)? {
        let has_docker = row.runtimes.iter().any(|v| v == "docker") && row.dockerfile.is_some();
        let has_apptainer =
            row.runtimes.iter().any(|v| v == "apptainer") && row.apptainer_def.is_some();
        let has_smoke = row.version_cmd.is_some();
        let pinned = row
            .pinned_commit
            .as_deref()
            .is_some_and(|s| s.len() == 40 && s.chars().all(|c| c.is_ascii_hexdigit()));
        println!(
            "{}\t{has_docker}\t{has_apptainer}\t{has_smoke}\t{pinned}",
            row.id
        );
    }
    Ok(())
}

/// # Errors
/// Returns an error if smoke script execution fails.
pub fn run_env_smoke(runtime: &str, tool: &str) -> Result<()> {
    run_smoke_script(runtime, tool)
}

fn normalize_stage_id(stage: &str) -> String {
    if stage.contains('.') {
        stage.to_string()
    } else {
        format!("fastq.{stage}")
    }
}

fn parse_registry(path: &Path) -> Result<String> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    Ok(raw)
}

/// # Errors
/// Returns an error if registry cannot be parsed.
pub fn registry_tools_for_stage(
    registry_path: &Path,
    stage: &str,
    kind: &str,
) -> Result<Vec<String>> {
    let parsed = parse_registry(registry_path)?;
    let stage_id = normalize_stage_id(stage);
    let Some(stage_entry) = parse_stage_registry_rows(&parsed)?
        .into_iter()
        .find(|entry| entry.id == stage_id)
    else {
        return Err(anyhow!("stage not found in registry: {stage_id}"));
    };

    let mut result = match kind {
        "primary" => stage_entry.primary_tools,
        "optional" => stage_entry.optional_alternatives,
        "validation" => stage_entry.validation_tools,
        "reporting" => stage_entry.reporting_tools,
        _ => {
            let mut all = Vec::new();
            all.extend(stage_entry.primary_tools);
            all.extend(stage_entry.optional_alternatives);
            all.extend(stage_entry.validation_tools);
            all.extend(stage_entry.reporting_tools);
            all
        }
    };
    result.sort();
    result.dedup();
    Ok(result)
}

/// # Errors
/// Returns an error if stage cannot be resolved.
pub fn run_env_smoke_for_stage(registry_path: &Path, runtime: &str, stage: &str) -> Result<()> {
    let tools = registry_tools_for_stage(registry_path, stage, "all")?;
    if tools.is_empty() {
        return Err(anyhow!("no tools found for stage {stage}"));
    }
    run_env_with_tools(runtime, &tools, "contract")
}

/// # Errors
/// Returns an error if prep script execution fails.
pub fn run_env_prep(
    registry_path: &Path,
    runtime: &str,
    tool: Option<&str>,
    stage: Option<&str>,
) -> Result<()> {
    if let Some(tool) = tool {
        return run_env_with_tools(runtime, &[tool.to_string()], "version");
    }
    if let Some(stage) = stage {
        let tools = registry_tools_for_stage(registry_path, stage, "all")?;
        if tools.is_empty() {
            return Err(anyhow!("no tools found for stage {stage}"));
        }
        return run_env_with_tools(runtime, &tools, "version");
    }
    run_env_with_tools(runtime, &[], "version")
}

fn run_env_with_tools(runtime: &str, tools: &[String], smoke_level: &str) -> Result<()> {
    run_smoke_script_batch(runtime, tools, smoke_level)
}

#[derive(Default, Serialize)]
struct RegistryRow {
    id: String,
    status: String,
    domain: Option<String>,
    domains: Vec<String>,
    stage_ids: Vec<String>,
    bindings: Vec<String>,
    tool_role: Option<String>,
    version: Option<String>,
    upstream: Option<String>,
    runtimes: Vec<String>,
    dockerfile: Option<String>,
    apptainer_def: Option<String>,
    version_cmd: Option<String>,
    help_cmd: Option<String>,
    expected_bin: Option<String>,
    pinned_commit: Option<String>,
    container_ref: Option<String>,
    expected_version_regex: Option<String>,
    healthcheck_cmd: Option<String>,
    smoke_version_cmd: Option<String>,
    smoke_help_cmd: Option<String>,
    smoke_require_help: Option<bool>,
    smoke_probes: Vec<String>,
    java_heap_mb: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct EnsureImagesReport {
    pub schema_version: &'static str,
    pub domain: String,
    pub stages: Vec<String>,
    pub tools: Vec<EnsureToolReport>,
    pub built: usize,
    pub reused: usize,
    pub quick_smoked: usize,
    pub failed: usize,
}

#[derive(Debug, Serialize)]
pub struct EnsureToolReport {
    pub tool_id: String,
    pub stage_id: String,
    pub sif_path: String,
    pub expected_digest: String,
    pub actual_digest: String,
    pub built: bool,
    pub quick_smoked: bool,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct SifInventoryReport {
    pub schema_version: &'static str,
    pub containers_dir: String,
    pub entries: Vec<SifInventoryEntry>,
}

#[derive(Debug, Serialize)]
pub struct SifInventoryEntry {
    pub tool_id: String,
    pub sif_path: String,
    pub sha256: String,
    pub smoke_manifest_path: Option<String>,
    pub smoke_status: Option<String>,
}

#[derive(Debug, Serialize)]
struct SmokeManifest {
    schema_version: &'static str,
    tool_id: String,
    stage_id: String,
    status: String,
    expected_digest: String,
    actual_digest: String,
    version_cmd: String,
    help_cmd: String,
    version: String,
    version_output_first_line: String,
    help_ok: bool,
    quick_smoke: bool,
    probe_commands: Vec<String>,
    java_heap_mb: Option<u64>,
    upstream: String,
    image_build_timestamp_unix_s: u64,
    checked_at_unix_s: u64,
}

fn parse_tools_registry_rows(raw: &str) -> Result<Vec<RegistryRow>> {
    let mut rows = Vec::new();
    let mut current: Option<RegistryRow> = None;

    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "[[tools]]" {
            if let Some(row) = current.take() {
                rows.push(row);
            }
            current = Some(RegistryRow::default());
            continue;
        }
        let Some(row) = current.as_mut() else {
            continue;
        };
        if let Some(value) = parse_toml_string(trimmed, "id") {
            row.id = value;
        } else if let Some(value) = parse_toml_string(trimmed, "status") {
            row.status = value;
        } else if let Some(value) = parse_toml_string(trimmed, "domain") {
            row.domain = Some(value);
        } else if let Some(values) = parse_toml_array(trimmed, "domains") {
            row.domains = values;
        } else if let Some(values) = parse_toml_array(trimmed, "stage_ids") {
            row.stage_ids = values;
        } else if let Some(values) = parse_toml_array(trimmed, "bindings") {
            row.bindings = values;
        } else if let Some(value) = parse_toml_string(trimmed, "tool_role") {
            row.tool_role = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "version") {
            row.version = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "upstream") {
            row.upstream = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "dockerfile") {
            row.dockerfile = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "apptainer_def") {
            row.apptainer_def = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "version_cmd") {
            row.version_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "help_cmd") {
            row.help_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "expected_bin") {
            row.expected_bin = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "pinned_commit") {
            row.pinned_commit = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "container_ref") {
            row.container_ref = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "expected_version_regex") {
            row.expected_version_regex = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "healthcheck_cmd") {
            row.healthcheck_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "smoke_version_cmd") {
            row.smoke_version_cmd = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "smoke_help_cmd") {
            row.smoke_help_cmd = Some(value);
        } else if let Some(value) = parse_toml_bool(trimmed, "smoke_require_help") {
            row.smoke_require_help = Some(value);
        } else if let Some(values) = parse_toml_array(trimmed, "smoke_probes") {
            row.smoke_probes = values;
        } else if let Some(value) = parse_toml_u64(trimmed, "java_heap_mb") {
            row.java_heap_mb = Some(value);
        } else if let Some(values) = parse_toml_array(trimmed, "runtimes") {
            row.runtimes = values;
        }
    }
    if let Some(row) = current {
        rows.push(row);
    }
    if rows.is_empty() {
        return Err(anyhow!("missing [[tools]] entries"));
    }
    Ok(rows)
}

