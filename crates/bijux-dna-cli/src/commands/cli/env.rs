use std::collections::{HashMap, HashSet};
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
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
        let version_cmd = tool.version_cmd.as_deref().unwrap_or("").trim();
        let help_cmd = tool.help_cmd.as_deref().unwrap_or("").trim();
        if version_cmd.is_empty() || help_cmd.is_empty() {
            offenders.push(format!(
                "tool={} missing smoke commands (version/help)",
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

/// # Errors
/// Returns an error if registry parsing, Apptainer build/smoke, or manifest writes fail.
#[allow(clippy::too_many_lines)]
pub fn ensure_apptainer_images(
    registry_path: &Path,
    domain: &str,
    stages_csv: &str,
    force_smoke: bool,
) -> Result<EnsureImagesReport> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?
        .into_iter()
        .map(|row| (row.id.clone(), row))
        .collect::<std::collections::BTreeMap<_, _>>();
    let stages = parse_stage_registry_rows(&raw)?
        .into_iter()
        .map(|row| (row.id.clone(), row))
        .collect::<std::collections::BTreeMap<_, _>>();
    let stage_ids = normalize_stage_ids(domain, stages_csv);

    let root = PathBuf::from("/home/bijan/bijux");
    let containers_root = root.join("bijux-dna-containers");
    let data_root = root.join("bijux-dna-data");
    let results_root = root.join("bijux-dna-results");
    bijux_dna_api::v1::api::run::ensure_dir(&containers_root)?;
    bijux_dna_api::v1::api::run::ensure_dir(&data_root)?;
    bijux_dna_api::v1::api::run::ensure_dir(&results_root)?;

    let mut reports = Vec::new();
    let mut built = 0usize;
    let mut reused = 0usize;
    let mut quick_smoked = 0usize;
    let mut failed = 0usize;
    let mut auto_demoted = Vec::new();

    for stage_id in &stage_ids {
        let Some(stage) = stages.get(stage_id) else {
            return Err(anyhow!("stage not found in registry: {stage_id}"));
        };
        let mut stage_tools = stage.primary_tools.clone();
        stage_tools.extend(stage.optional_alternatives.clone());
        stage_tools.extend(stage.validation_tools.clone());
        stage_tools.extend(stage.reporting_tools.clone());
        stage_tools.sort();
        stage_tools.dedup();

        for tool_id in stage_tools {
            let Some(tool) = tools.get(&tool_id) else {
                continue;
            };
            let has_apptainer = tool.runtimes.iter().any(|runtime| runtime == "apptainer");
            if !has_apptainer {
                continue;
            }
            let Some(def_rel) = tool.apptainer_def.as_deref() else {
                continue;
            };
            let expected_digest = expected_registry_digest(tool)
                .ok_or_else(|| anyhow!("tool {tool_id} is missing sha256 pin in registry"))?;
            let tool_dir = containers_root.join(&tool_id);
            bijux_dna_api::v1::api::run::ensure_dir(&tool_dir)?;
            let sif_path = tool_dir.join(format!("{expected_digest}.sif"));
            let smoke_manifest_path = tool_dir.join("smoke_manifest.json");
            let mut built_this = false;

            if sif_path.exists() {
                let actual = hash_file_sha256_hex(&sif_path)?;
                if actual != expected_digest {
                    std::fs::remove_file(&sif_path)
                        .with_context(|| format!("remove {}", sif_path.display()))?;
                }
            }
            if sif_path.exists() {
                reused += 1;
            } else {
                let def_path = std::env::current_dir()?.join(def_rel);
                build_apptainer_image(&def_path, &sif_path)?;
                built_this = true;
                built += 1;
            }

            let actual_digest = hash_file_sha256_hex(&sif_path)?;
            if actual_digest != expected_digest {
                return Err(anyhow!(
                    "digest mismatch for {tool_id}: expected {expected_digest}, got {actual_digest}"
                ));
            }

            let do_quick_smoke = force_smoke || should_run_weekly_quick_smoke(&smoke_manifest_path);
            let version_cmd = tool
                .version_cmd
                .clone()
                .unwrap_or_else(|| format!("{tool_id} --version"));
            let help_cmd = tool
                .help_cmd
                .clone()
                .unwrap_or_else(|| format!("{tool_id} --help"));
            let mut status = "ok".to_string();
            if do_quick_smoke {
                quick_smoked += 1;
                let smoke = run_smoke_with_manifest(
                    &sif_path,
                    &tool_id,
                    stage_id,
                    &expected_digest,
                    &actual_digest,
                    &version_cmd,
                    &help_cmd,
                    &data_root,
                    &results_root,
                )?;
                if smoke.status != "ok" {
                    status = "wrapper_failed_auto_demoted".to_string();
                    failed += 1;
                    auto_demoted.push(tool_id.clone());
                }
                bijux_dna_infra::atomic_write_json(&smoke_manifest_path, &smoke)
                    .with_context(|| format!("write {}", smoke_manifest_path.display()))?;
            }

            reports.push(EnsureToolReport {
                tool_id: tool_id.clone(),
                stage_id: stage_id.clone(),
                sif_path: sif_path.display().to_string(),
                expected_digest: expected_digest.clone(),
                actual_digest,
                built: built_this,
                quick_smoked: do_quick_smoke,
                status,
            });
        }
    }

    if !auto_demoted.is_empty() {
        let payload = serde_json::json!({
            "schema_version": "bijux.apptainer_auto_demote.v1",
            "tools": auto_demoted,
            "updated_at_unix_s": now_unix_s(),
            "reason": "help/version smoke failure",
        });
        let path = containers_root.join("auto_demoted_tools.json");
        bijux_dna_infra::atomic_write_json(&path, &payload)
            .with_context(|| format!("write {}", path.display()))?;
    }

    Ok(EnsureImagesReport {
        schema_version: "bijux.apptainer.ensure_images.v1",
        domain: domain.to_string(),
        stages: stage_ids,
        tools: reports,
        built,
        reused,
        quick_smoked,
        failed,
    })
}

/// # Errors
/// Returns an error if registry cannot be read.
pub fn print_registry_list_tools(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let mut tools = parse_tools_registry_rows(&raw)?
        .into_iter()
        .filter(|row| row.status != "planned" && row.status != "out_of_scope")
        .map(|row| row.id)
        .collect::<Vec<_>>();
    tools.sort();
    tools.dedup();
    for tool in tools {
        println!("{tool}");
    }
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read.
pub fn print_registry_tools(registry_path: &Path, stage: Option<&str>, kind: &str) -> Result<()> {
    if let Some(stage) = stage {
        let tools = registry_tools_for_stage(registry_path, stage, kind)?;
        println!("{}", tools.join(","));
        return Ok(());
    }
    print_registry_list_tools(registry_path)
}

/// # Errors
/// Returns an error if registry cannot be read.
pub fn print_registry_list_stages(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let mut stages = parse_stage_registry_rows(&raw)?
        .into_iter()
        .map(|stage| stage.id)
        .collect::<Vec<_>>();
    stages.sort();
    for stage in stages {
        println!("{stage}");
    }
    Ok(())
}

/// # Errors
/// Returns an error if id is not found or registry cannot be parsed.
pub fn print_registry_show(registry_path: &Path, id: &str) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    if let Some(tool) = parse_tools_registry_rows(&raw)?
        .into_iter()
        .find(|tool| tool.id == id)
    {
        crate::commands::cli::render::json::print_pretty(&serde_json::json!({
            "id": tool.id,
            "version": tool.version,
            "upstream": tool.upstream,
            "runtimes": tool.runtimes,
            "dockerfile": tool.dockerfile,
            "apptainer_def": tool.apptainer_def,
            "version_cmd": tool.version_cmd,
            "help_cmd": tool.help_cmd,
            "healthcheck_cmd": tool.healthcheck_cmd,
            "expected_bin": tool.expected_bin,
            "expected_version_regex": tool.expected_version_regex,
            "pinned_commit": tool.pinned_commit,
        }))?;
        return Ok(());
    }
    if let Some(stage) = parse_stage_registry_rows(&raw)?
        .into_iter()
        .find(|stage| stage.id == id)
    {
        crate::commands::cli::render::json::print_pretty(&serde_json::json!({
            "id": stage.id,
            "primary_tools": stage.primary_tools,
            "optional_alternatives": stage.optional_alternatives,
            "validation_tools": stage.validation_tools,
            "reporting_tools": stage.reporting_tools,
        }))?;
        return Ok(());
    }
    Err(anyhow!("registry id not found: {id}"))
}

/// # Errors
/// Returns an error if id is not found or registry cannot be parsed.
pub fn print_registry_show_tool(registry_path: &Path, id: &str) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let Some(tool) = parse_tools_registry_rows(&raw)?
        .into_iter()
        .find(|tool| tool.id == id)
    else {
        return Err(anyhow!("tool not found in registry: {id}"));
    };
    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "id": tool.id,
        "version": tool.version,
        "upstream": tool.upstream,
        "runtimes": tool.runtimes,
        "dockerfile": tool.dockerfile,
        "apptainer_def": tool.apptainer_def,
        "version_cmd": tool.version_cmd,
        "help_cmd": tool.help_cmd,
        "healthcheck_cmd": tool.healthcheck_cmd,
        "expected_bin": tool.expected_bin,
        "expected_version_regex": tool.expected_version_regex,
        "pinned_commit": tool.pinned_commit,
    }))?;
    Ok(())
}

/// # Errors
/// Returns an error if tool cannot be resolved from registry.
pub fn verify_registry_tool(registry_path: &Path, id: &str) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let Some(tool) = parse_tools_registry_rows(&raw)?
        .into_iter()
        .find(|tool| tool.id == id)
    else {
        return Err(anyhow!("tool not found in registry: {id}"));
    };
    let pin = tool
        .pinned_commit
        .clone()
        .unwrap_or_else(|| "missing".to_string());
    let version_cmd = tool.version_cmd.clone().unwrap_or_default();
    let help_cmd = tool.help_cmd.clone().unwrap_or_default();
    let healthcheck_cmd = tool
        .healthcheck_cmd
        .clone()
        .unwrap_or_else(|| help_cmd.clone());
    let expected_version_regex = tool
        .expected_version_regex
        .clone()
        .unwrap_or_else(|| "v?[0-9]+\\.[0-9]+([.-][0-9A-Za-z]+)?".to_string());
    let version_output =
        run_shell_capture(&version_cmd).unwrap_or_else(|err| format!("error:{err}"));
    let help_output = run_shell_capture(&help_cmd).unwrap_or_else(|err| format!("error:{err}"));
    let health_output =
        run_shell_capture(&healthcheck_cmd).unwrap_or_else(|err| format!("error:{err}"));
    let version_matches_regex = Regex::new(&expected_version_regex)
        .ok()
        .is_some_and(|regex| regex.is_match(&version_output));
    let parsed_version =
        parse_first_version(&version_output).unwrap_or_else(|| "unknown".to_string());

    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "tool_id": tool.id,
        "pin": pin,
        "entrypoint": tool.expected_bin,
        "version_cmd": version_cmd,
        "help_cmd": help_cmd,
        "healthcheck_cmd": healthcheck_cmd,
        "expected_version_regex": expected_version_regex,
        "version_output_parse": parsed_version,
        "version_output_matches_regex": version_matches_regex,
        "version_output_sample": version_output.lines().next().unwrap_or(""),
        "help_ok": !help_output.starts_with("error:"),
        "healthcheck_ok": !health_output.starts_with("error:"),
    }))?;
    Ok(())
}

fn parse_first_version(output: &str) -> Option<String> {
    let mut chars = output.chars().peekable();
    let mut token = String::new();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() {
            token.push(ch);
            while let Some(next) = chars.peek() {
                if next.is_ascii_digit() || *next == '.' || *next == '-' {
                    token.push(*next);
                    let _ = chars.next();
                } else {
                    break;
                }
            }
            if token.contains('.') {
                return Some(token);
            }
            token.clear();
        }
    }
    None
}

/// # Errors
/// Returns an error if id is not found or registry cannot be parsed.
pub fn print_registry_show_stage(registry_path: &Path, id: &str) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let Some(stage) = parse_stage_registry_rows(&raw)?
        .into_iter()
        .find(|stage| stage.id == id)
    else {
        return Err(anyhow!("stage not found in registry: {id}"));
    };
    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "id": stage.id,
        "primary_tools": stage.primary_tools,
        "optional_alternatives": stage.optional_alternatives,
        "validation_tools": stage.validation_tools,
        "reporting_tools": stage.reporting_tools,
    }))?;
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn print_registry_export_json(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let mut tools = parse_tools_registry_rows(&raw)?;
    let mut stages = parse_stage_registry_rows(&raw)?;
    tools.sort_by(|a, b| a.id.cmp(&b.id));
    stages.sort_by(|a, b| a.id.cmp(&b.id));
    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "schema_version": "bijux.registry_export.v1",
        "tools": tools,
        "stages": stages
    }))?;
    Ok(())
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn print_registry_coverage_matrix(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let tools = parse_tools_registry_rows(&raw)?
        .into_iter()
        .map(|row| (row.id.clone(), row))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut stages = parse_stage_registry_rows(&raw)?;
    stages.sort_by(|a, b| a.id.cmp(&b.id));
    let mut rows = Vec::new();
    for stage in stages {
        let stage_id = stage.id.clone();
        let mut stage_tools = stage.primary_tools.clone();
        stage_tools.extend(stage.optional_alternatives);
        stage_tools.extend(stage.validation_tools);
        stage_tools.extend(stage.reporting_tools);
        stage_tools.sort();
        stage_tools.dedup();
        for tool_id in stage_tools {
            let Some(tool) = tools.get(&tool_id) else {
                continue;
            };
            rows.push(serde_json::json!({
                "stage_id": stage_id,
                "tool_id": tool_id,
                "status": tool.status,
                "runtimes": tool.runtimes,
            }));
        }
    }
    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "schema_version": "bijux.registry.coverage_matrix.v1",
        "rows": rows
    }))?;
    Ok(())
}

#[derive(Default, Serialize)]
struct StageRegistryRow {
    id: String,
    required_tool_roles: Vec<String>,
    primary_tools: Vec<String>,
    optional_alternatives: Vec<String>,
    validation_tools: Vec<String>,
    reporting_tools: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PolicyCheckResult {
    pub name: String,
    pub ok: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
pub struct PolicyCleanReport {
    pub schema_version: &'static str,
    pub domain: String,
    pub ok: bool,
    pub checks: Vec<PolicyCheckResult>,
}

/// # Errors
/// Returns an error if registry cannot be read or parsed.
pub fn print_env_export_json(registry_path: &Path) -> Result<()> {
    let raw = std::fs::read_to_string(registry_path)
        .with_context(|| format!("read {}", registry_path.display()))?;
    let mut tools = parse_tools_registry_rows(&raw)?;
    tools.sort_by(|a, b| a.id.cmp(&b.id));
    let payload = tools
        .into_iter()
        .map(|row| {
            let has_docker = row.runtimes.iter().any(|v| v == "docker") && row.dockerfile.is_some();
            let has_apptainer =
                row.runtimes.iter().any(|v| v == "apptainer") && row.apptainer_def.is_some();
            serde_json::json!({
                "id": row.id,
                "status": row.status,
                "version": row.version,
                "upstream": row.upstream,
                "runtimes": row.runtimes,
                "dockerfile": row.dockerfile,
                "apptainer_def": row.apptainer_def,
                "version_cmd": row.version_cmd,
                "help_cmd": row.help_cmd,
                "expected_bin": row.expected_bin,
                "pinned_commit": row.pinned_commit,
                "has_docker": has_docker,
                "has_apptainer": has_apptainer,
                "has_smoke": row.version_cmd.is_some(),
                "platforms": ["linux/arm64", "linux/amd64"]
            })
        })
        .collect::<Vec<_>>();
    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "schema_version": "bijux.environment_export.v1",
        "tools": payload
    }))?;
    Ok(())
}

fn normalize_stage_ids(domain: &str, stages_csv: &str) -> Vec<String> {
    let mut stage_ids = stages_csv
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(|item| {
            if item.contains('.') {
                item.to_string()
            } else {
                format!("{domain}.{item}")
            }
        })
        .collect::<Vec<_>>();
    stage_ids.sort();
    stage_ids.dedup();
    stage_ids
}

fn expected_registry_digest(tool: &RegistryRow) -> Option<String> {
    let pin = tool.pinned_commit.as_deref().unwrap_or("");
    if let Some(digest) = pin.strip_prefix("sha256:") {
        return Some(digest.to_string());
    }
    let container_ref = tool.container_ref.as_deref().unwrap_or("");
    container_ref
        .split("@sha256:")
        .nth(1)
        .map(std::string::ToString::to_string)
}

fn build_apptainer_image(def_path: &Path, sif_path: &Path) -> Result<()> {
    if let Some(parent) = sif_path.parent() {
        bijux_dna_api::v1::api::run::ensure_dir(parent)?;
    }
    let output = Command::new("apptainer")
        .arg("build")
        .arg("--force")
        .arg(sif_path)
        .arg(def_path)
        .output()
        .with_context(|| format!("build apptainer image from {}", def_path.display()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "apptainer build failed for {}: {}",
            def_path.display(),
            stderr.trim()
        ));
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_smoke_with_manifest(
    sif_path: &Path,
    tool_id: &str,
    stage_id: &str,
    expected_digest: &str,
    actual_digest: &str,
    version_cmd: &str,
    help_cmd: &str,
    data_root: &Path,
    results_root: &Path,
) -> Result<SmokeManifest> {
    let version_out = run_apptainer_exec(sif_path, version_cmd, data_root, results_root)?;
    let help_ok = run_apptainer_exec(sif_path, help_cmd, data_root, results_root).is_ok();
    let parsed_version = parse_first_version(&version_out).unwrap_or_default();
    let fallback_version = version_out
        .lines()
        .next()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .unwrap_or("n/a")
        .to_string();
    let status = if help_ok && !parsed_version.is_empty() {
        "ok"
    } else {
        "wrapper_failed"
    };
    Ok(SmokeManifest {
        schema_version: "bijux.apptainer.smoke_manifest.v1",
        tool_id: tool_id.to_string(),
        stage_id: stage_id.to_string(),
        status: status.to_string(),
        expected_digest: expected_digest.to_string(),
        actual_digest: actual_digest.to_string(),
        version_cmd: version_cmd.to_string(),
        help_cmd: help_cmd.to_string(),
        version: if parsed_version.is_empty() {
            fallback_version
        } else {
            parsed_version
        },
        version_output_first_line: version_out.lines().next().unwrap_or("").to_string(),
        help_ok,
        quick_smoke: true,
        checked_at_unix_s: now_unix_s(),
    })
}

fn run_apptainer_exec(
    sif_path: &Path,
    command: &str,
    data_root: &Path,
    results_root: &Path,
) -> Result<String> {
    let input_bind = format!("{}:/bijux/input:ro", data_root.display());
    let output_bind = format!("{}:/bijux/output:rw", results_root.display());
    let db_bind = format!("{}:/bijux/db:ro", data_root.join("banks").display());
    let output = Command::new("apptainer")
        .arg("exec")
        .arg("--containall")
        .arg("--cleanenv")
        .arg("--bind")
        .arg(input_bind)
        .arg("--bind")
        .arg(output_bind)
        .arg("--bind")
        .arg(db_bind)
        .arg(sif_path)
        .arg("sh")
        .arg("-lc")
        .arg(command)
        .output()
        .with_context(|| format!("apptainer exec {}", sif_path.display()))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if output.status.success() {
        if stdout.trim().is_empty() {
            Ok(stderr)
        } else {
            Ok(stdout)
        }
    } else {
        Err(anyhow!(
            "apptainer exec failed for {}: {}",
            sif_path.display(),
            stderr.trim()
        ))
    }
}

fn hash_file_sha256_hex(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0_u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .with_context(|| format!("read {}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn should_run_weekly_quick_smoke(manifest_path: &Path) -> bool {
    let Ok(meta) = std::fs::metadata(manifest_path) else {
        return true;
    };
    let Ok(modified) = meta.modified() else {
        return true;
    };
    let Ok(age) = SystemTime::now().duration_since(modified) else {
        return true;
    };
    age >= Duration::from_secs(7 * 24 * 3600)
}

fn now_unix_s() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_or(0, |dur| dur.as_secs())
}

fn parse_stage_registry_rows(raw: &str) -> Result<Vec<StageRegistryRow>> {
    let mut rows = Vec::new();
    let mut current: Option<StageRegistryRow> = None;
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if trimmed == "[[stages]]" {
            if let Some(row) = current.take() {
                rows.push(row);
            }
            current = Some(StageRegistryRow::default());
            continue;
        }
        let Some(row) = current.as_mut() else {
            continue;
        };
        if let Some(value) = parse_toml_string(trimmed, "id") {
            row.id = value;
        } else if let Some(values) = parse_toml_array(trimmed, "required_tool_roles") {
            row.required_tool_roles = values;
        } else if let Some(values) = parse_toml_array(trimmed, "primary_tools") {
            row.primary_tools = values;
        } else if let Some(values) = parse_toml_array(trimmed, "optional_alternatives") {
            row.optional_alternatives = values;
        } else if let Some(values) = parse_toml_array(trimmed, "validation_tools") {
            row.validation_tools = values;
        } else if let Some(values) = parse_toml_array(trimmed, "reporting_tools") {
            row.reporting_tools = values;
        }
    }
    if let Some(row) = current {
        rows.push(row);
    }
    if rows.is_empty() {
        return Err(anyhow!("missing [[stages]] entries"));
    }
    Ok(rows)
}

fn parse_toml_string(line: &str, key: &str) -> Option<String> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }
    let value = rhs.trim();
    if !(value.starts_with('"') && value.ends_with('"') && value.len() >= 2) {
        return None;
    }
    Some(value[1..value.len() - 1].to_string())
}

fn parse_toml_array(line: &str, key: &str) -> Option<Vec<String>> {
    let (lhs, rhs) = line.split_once('=')?;
    if lhs.trim() != key {
        return None;
    }
    let value = rhs.trim();
    if !(value.starts_with('[') && value.ends_with(']') && value.len() >= 2) {
        return None;
    }
    let inner = &value[1..value.len() - 1];
    let items = inner
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(|token| token.trim_matches('"').to_string())
        .collect::<Vec<_>>();
    Some(items)
}

pub fn print_env_info<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) {
    println!("platform: {}", platform.name);
    println!("runner: {}", platform.runner);
    println!("image count: {}", catalog.len());
    println!("cache: {}", cache_dir(platform.runner).to_string_lossy());
}

pub fn env_doctor<S: ::std::hash::BuildHasher>(
    catalog: &HashMap<String, ToolImageSpec, S>,
    platform: &PlatformSpec,
) {
    println!("bijux dna env doctor");
    let runners = available_runners().unwrap_or_default();
    print_check(
        "cache directory writable",
        ensure_cache_writable(platform.runner),
    );
    print_check("runner available", runners.contains(&platform.runner));
    println!("runners: {}", display_runners(&runners));
    for (tool, spec) in catalog {
        let Ok(image) = resolve_image(spec, platform) else {
            continue;
        };
        let exists = docker_image_exists(&image);
        print_check(&format!("image {tool}"), exists);
    }
}

fn ensure_cache_writable(runner: RuntimeKind) -> bool {
    let cache_dir = cache_dir(runner);
    bijux_dna_api::v1::api::run::ensure_dir(&cache_dir).is_ok()
}

fn print_check(name: &str, ok: bool) {
    if ok {
        println!("ok   {name}");
    } else {
        println!("fail {name}");
    }
}

fn display_runners(runners: &[RuntimeKind]) -> String {
    runners
        .iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct HomeGuard {
        original: Option<std::ffi::OsString>,
    }

    impl Drop for HomeGuard {
        fn drop(&mut self) {
            if let Some(value) = self.original.take() {
                std::env::set_var("HOME", value);
            } else {
                std::env::remove_var("HOME");
            }
        }
    }

    #[test]
    fn display_runners_is_deterministic() {
        let runners = vec![RuntimeKind::Docker, RuntimeKind::Apptainer];
        assert_eq!(display_runners(&runners), "docker, apptainer");
    }

    #[test]
    fn ensure_cache_writable_uses_home() -> anyhow::Result<()> {
        let temp = bijux_dna_api::v1::api::run::temp_dir("bijux")?;
        let original_home = std::env::var_os("HOME");
        let _guard = HomeGuard {
            original: original_home,
        };
        std::env::set_var("HOME", temp.path());
        assert!(ensure_cache_writable(RuntimeKind::Docker));
        Ok(())
    }
}
