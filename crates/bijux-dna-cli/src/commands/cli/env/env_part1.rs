use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::Path;
use std::process::Command;
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

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn print_registry_audit_fix_suggestions(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?;
    for tool in tools {
        let mut suggestions = Vec::new();
        if tool.bindings.is_empty() {
            if tool.stage_ids.is_empty() {
                suggestions.push("bindings = [\"<domain.stage>\"]".to_string());
            } else {
                suggestions.push(format!("bindings = {}", toml_array_inline(&tool.stage_ids)));
            }
        }
        if tool.domains.is_empty() {
            let mut domains = tool
                .bindings
                .iter()
                .chain(tool.stage_ids.iter())
                .filter_map(|stage_id| stage_id.split('.').next().map(str::to_string))
                .collect::<Vec<_>>();
            domains.sort();
            domains.dedup();
            if !domains.is_empty() {
                suggestions.push(format!("domains = {}", toml_array_inline(&domains)));
            }
        }
        if tool.tool_role.as_deref().unwrap_or("").trim().is_empty() {
            suggestions.push("tool_role = \"<aligner|screen|trimmer|qc|filter|validator|merger|corrector|transform>\"".to_string());
        }
        if let Some(domain) = tool.domain.clone().filter(|_| !tool.bindings.is_empty()) {
            let mismatch = tool.bindings.iter().any(|stage_id| {
                stage_id
                    .split('.')
                    .next()
                    .is_some_and(|d| d != domain.as_str())
            });
            if mismatch {
                suggestions.push(format!("# domain mismatch: current domain = \"{domain}\""));
            }
        }
        if suggestions.is_empty() {
            continue;
        }
        println!("[[tools]] # id = \"{}\"", tool.id);
        for line in suggestions {
            println!("{line}");
        }
        println!();
    }
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn registry_binding_violations(
    registry_path: &Path,
    domain: Option<&str>,
) -> Result<Vec<String>> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?;
    let stages = parse_stage_registry_rows(&raw)?;
    let stage_ids = stages.into_iter().map(|row| row.id).collect::<HashSet<_>>();

    let mut offenders = Vec::new();
    for tool in tools {
        if tool.id.is_empty() || tool.status == "planned" || tool.status == "out_of_scope" {
            continue;
        }
        let bindings = if tool.bindings.is_empty() {
            tool.stage_ids.clone()
        } else {
            tool.bindings.clone()
        };
        if bindings.is_empty() {
            offenders.push(format!("tool={} missing non-empty bindings", tool.id));
            continue;
        }
        if let Some(dom) = domain {
            let relevant = bindings
                .iter()
                .any(|stage_id| stage_id.starts_with(&format!("{dom}.")));
            if !relevant {
                continue;
            }
        }
        for stage_id in &bindings {
            if !stage_ids.contains(stage_id) {
                offenders.push(format!(
                    "tool={} binding references unknown stage {}",
                    tool.id, stage_id
                ));
            }
            if let Some(dom) = domain {
                if !stage_id.starts_with(&format!("{dom}.")) {
                    offenders.push(format!(
                        "tool={} binding {} crosses requested domain {}",
                        tool.id, stage_id, dom
                    ));
                }
            }
        }
    }

    offenders.sort();
    offenders.dedup();
    Ok(offenders)
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn print_registry_binding_violations(registry_path: &Path, domain: Option<&str>) -> Result<()> {
    let offenders = registry_binding_violations(registry_path, domain)?;
    if offenders.is_empty() {
        println!("binding_violations: none");
        return Ok(());
    }
    for offender in offenders {
        println!("{offender}");
    }
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn policy_clean_report(registry_path: &Path, domain: &str) -> Result<PolicyCleanReport> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?;
    let stages = parse_stage_registry_rows(&raw)?;
    let tool_by_id = tools
        .iter()
        .map(|tool| (tool.id.clone(), tool))
        .collect::<std::collections::BTreeMap<_, _>>();

    let binding_offenders = registry_binding_violations(registry_path, Some(domain))?;

    let role_offenders = role_policy_offenders(&stages, &tool_by_id, domain);
    let smoke_offenders = smoke_policy_offenders(tools, domain);

    let checks = vec![
        PolicyCheckResult {
            name: "binding_policy".to_string(),
            ok: binding_offenders.is_empty(),
            detail: if binding_offenders.is_empty() {
                "ok".to_string()
            } else {
                binding_offenders.join("; ")
            },
        },
        PolicyCheckResult {
            name: "role_policy".to_string(),
            ok: role_offenders.is_empty(),
            detail: if role_offenders.is_empty() {
                "ok".to_string()
            } else {
                role_offenders.join("; ")
            },
        },
        PolicyCheckResult {
            name: "smoke_policy".to_string(),
            ok: smoke_offenders.is_empty(),
            detail: if smoke_offenders.is_empty() {
                "ok".to_string()
            } else {
                smoke_offenders.join("; ")
            },
        },
    ];
    let ok = checks.iter().all(|check| check.ok);
    Ok(PolicyCleanReport {
        schema_version: "bijux.policy.clean.v1",
        domain: domain.to_string(),
        ok,
        checks,
    })
}

fn role_policy_offenders(
    stages: &[StageRegistryRow],
    tool_by_id: &std::collections::BTreeMap<String, &RegistryRow>,
    domain: &str,
) -> Vec<String> {
    let mut offenders = Vec::new();
    for stage in stages {
        if !stage.id.starts_with(&format!("{domain}.")) {
            continue;
        }
        let required = stage
            .required_tool_roles
            .iter()
            .map(String::as_str)
            .collect::<HashSet<_>>();
        if required.is_empty() {
            offenders.push(format!("stage={} missing required_tool_roles", stage.id));
            continue;
        }
        for tool_id in stage_tool_ids(stage) {
            match tool_by_id.get(&tool_id) {
                Some(tool) if required.contains(tool.tool_role.as_deref().unwrap_or("")) => {}
                Some(tool) => offenders.push(format!(
                    "stage={} tool={} role={} not in {:?}",
                    stage.id,
                    tool_id,
                    tool.tool_role.as_deref().unwrap_or(""),
                    stage.required_tool_roles
                )),
                None => offenders.push(format!("stage={} unknown tool={tool_id}", stage.id)),
            }
        }
    }
    offenders.sort();
    offenders.dedup();
    offenders
}

fn stage_tool_ids(stage: &StageRegistryRow) -> Vec<String> {
    let mut ids = stage.primary_tools.clone();
    ids.extend(stage.optional_alternatives.clone());
    ids.extend(stage.validation_tools.clone());
    ids.extend(stage.reporting_tools.clone());
    ids.sort();
    ids.dedup();
    ids
}

fn smoke_policy_offenders(tools: Vec<RegistryRow>, domain: &str) -> Vec<String> {
    let mut offenders = Vec::new();
    for tool in tools {
        if tool.status != "supported" || !tool_in_domain(&tool, domain) {
            continue;
        }
        let version_cmd = tool
            .smoke_version_cmd
            .as_deref()
            .or(tool.version_cmd.as_deref())
            .unwrap_or("")
            .trim();
        let help_cmd = tool
            .smoke_help_cmd
            .as_deref()
            .or(tool.help_cmd.as_deref())
            .unwrap_or("")
            .trim();
        let require_help = tool.smoke_require_help.unwrap_or(true);
        if version_cmd.is_empty() || (require_help && help_cmd.is_empty()) {
            offenders.push(format!(
                "tool={} missing smoke commands (version/help policy)",
                tool.id
            ));
        }
    }
    offenders.sort();
    offenders.dedup();
    offenders
}

fn tool_in_domain(tool: &RegistryRow, domain: &str) -> bool {
    tool.bindings
        .iter()
        .chain(tool.stage_ids.iter())
        .any(|stage| stage.starts_with(&format!("{domain}.")))
}

/// # Errors
/// Returns an error if HPC registry lint fails.
pub fn lint_registry_hpc(
    cwd: &Path,
    registry_path: &Path,
    domain: Option<&str>,
    stages_csv: Option<&str>,
) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?
        .into_iter()
        .map(|row| (row.id.clone(), row))
        .collect::<HashMap<_, _>>();
    let mut stages = parse_stage_registry_rows(&raw)?;
    if let Some(csv) = stages_csv {
        let normalized = normalize_stage_ids(domain.unwrap_or("fastq"), csv);
        stages.retain(|row| normalized.contains(&row.id));
    } else if let Some(dom) = domain {
        let prefix = format!("{dom}.");
        stages.retain(|row| row.id.starts_with(&prefix));
    }

    let mut offenders = Vec::new();
    for stage in stages {
        let mut stage_tools = stage.primary_tools.clone();
        stage_tools.extend(stage.optional_alternatives);
        stage_tools.extend(stage.validation_tools);
        stage_tools.extend(stage.reporting_tools);
        stage_tools.sort();
        stage_tools.dedup();
        for tool_id in stage_tools {
            let Some(tool) = tools.get(&tool_id) else {
                offenders.push(format!(
                    "stage={} tool={} missing [[tools]] row",
                    stage.id, tool_id
                ));
                continue;
            };
            let Some(def_rel) = tool.apptainer_def.as_deref() else {
                offenders.push(format!(
                    "stage={} tool={} missing apptainer_def",
                    stage.id, tool_id
                ));
                continue;
            };
            let def_path = cwd.join(def_rel);
            if !def_path.exists() {
                offenders.push(format!(
                    "stage={} tool={} apptainer_def missing at {}",
                    stage.id,
                    tool_id,
                    def_path.display()
                ));
                continue;
            }
            let raw_def = std::fs::read_to_string(&def_path)
                .with_context(|| format!("read {}", def_path.display()))?;
            if !raw_def
                .lines()
                .any(|line| line.trim_start().starts_with("Bootstrap:"))
            {
                offenders.push(format!(
                    "stage={} tool={} apptainer_def missing Bootstrap header",
                    stage.id, tool_id
                ));
            }
            if !raw_def.contains("%post") {
                offenders.push(format!(
                    "stage={} tool={} apptainer_def missing %post section",
                    stage.id, tool_id
                ));
            }
        }
    }
    if !offenders.is_empty() {
        return Err(anyhow!(
            "registry lint --hpc failed:\n{}",
            offenders.join("\n")
        ));
    }
    println!("registry lint --hpc: ok");
    Ok(())
}

fn toml_array_inline(values: &[String]) -> String {
    format!(
        "[{}]",
        values
            .iter()
            .map(|value| format!("\"{value}\""))
            .collect::<Vec<_>>()
            .join(", ")
    )
}

