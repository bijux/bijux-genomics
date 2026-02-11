use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::env::{
    available_runners, cache_dir, docker_image_exists, resolve_image, run_smoke_script,
    run_smoke_script_batch, PlatformSpec, RuntimeKind, ToolImageSpec,
};
use serde::Serialize;

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
    version: Option<String>,
    upstream: Option<String>,
    runtimes: Vec<String>,
    dockerfile: Option<String>,
    apptainer_def: Option<String>,
    version_cmd: Option<String>,
    help_cmd: Option<String>,
    expected_bin: Option<String>,
    pinned_commit: Option<String>,
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
            "expected_bin": tool.expected_bin,
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
        "expected_bin": tool.expected_bin,
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
    let version_output =
        run_shell_capture(&version_cmd).unwrap_or_else(|err| format!("error:{err}"));
    let help_output = run_shell_capture(&help_cmd).unwrap_or_else(|err| format!("error:{err}"));
    let parsed_version =
        parse_first_version(&version_output).unwrap_or_else(|| "unknown".to_string());

    crate::commands::cli::render::json::print_pretty(&serde_json::json!({
        "tool_id": tool.id,
        "pin": pin,
        "entrypoint": tool.expected_bin,
        "version_cmd": version_cmd,
        "help_cmd": help_cmd,
        "version_output_parse": parsed_version,
        "version_output_sample": version_output.lines().next().unwrap_or(""),
        "help_ok": !help_output.starts_with("error:"),
    }))?;
    Ok(())
}

fn run_shell_capture(cmd: &str) -> Result<String> {
    if cmd.trim().is_empty() {
        return Err(anyhow!("empty command"));
    }
    let output = Command::new("sh")
        .arg("-lc")
        .arg(cmd)
        .output()
        .with_context(|| format!("execute `{cmd}`"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let merged = if stdout.trim().is_empty() {
        stderr
    } else {
        stdout
    };
    if output.status.success() {
        Ok(merged)
    } else {
        Err(anyhow!("{merged}"))
    }
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
    primary_tools: Vec<String>,
    optional_alternatives: Vec<String>,
    validation_tools: Vec<String>,
    reporting_tools: Vec<String>,
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
