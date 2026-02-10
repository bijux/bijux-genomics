use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::env::{
    available_runners, cache_dir, docker_image_exists, resolve_image, run_smoke_script,
    PlatformSpec, RuntimeKind, ToolImageSpec,
};

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

fn parse_registry(path: &Path) -> Result<toml::Value> {
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    raw.parse().context("parse registry toml")
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
    let stages = parsed
        .get("stages")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let Some(stage_entry) = stages.iter().find(|entry| {
        entry
            .get("id")
            .and_then(toml::Value::as_str)
            .is_some_and(|id| id == stage_id)
    }) else {
        return Err(anyhow!("stage not found in registry: {stage_id}"));
    };

    let read = |key: &str| -> Vec<String> {
        stage_entry
            .get(key)
            .and_then(toml::Value::as_array)
            .map(|arr| {
                arr.iter()
                    .filter_map(toml::Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    let mut result = match kind {
        "primary" => read("primary_tools"),
        "optional" => read("optional_alternatives"),
        "validation" => read("validation_tools"),
        "reporting" => read("reporting_tools"),
        _ => {
            let mut all = Vec::new();
            all.extend(read("primary_tools"));
            all.extend(read("optional_alternatives"));
            all.extend(read("validation_tools"));
            all.extend(read("reporting_tools"));
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
    let script = match runtime {
        "docker-arm64" => "scripts/smoke-containers-docker-arm64.sh",
        "docker-amd64" => "scripts/smoke-containers-docker-amd64.sh",
        "apptainer" => "scripts/smoke-containers-apptainer.sh",
        other => {
            return Err(anyhow!(
                "unsupported runtime `{other}`; expected docker-arm64 | docker-amd64 | apptainer"
            ));
        }
    };
    let tools_csv = tools.join(",");
    let status = Command::new("sh")
        .arg(script)
        .env("TOOLS", tools_csv)
        .env("JOBS", "1")
        .env("SMOKE_LEVEL", smoke_level)
        .status()?;
    if !status.success() {
        return Err(anyhow!(
            "environment command failed for runtime={runtime} (exit={status})"
        ));
    }
    Ok(())
}

#[derive(Default)]
struct RegistryRow {
    id: String,
    runtimes: Vec<String>,
    dockerfile: Option<String>,
    apptainer_def: Option<String>,
    version_cmd: Option<String>,
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
        } else if let Some(value) = parse_toml_string(trimmed, "dockerfile") {
            row.dockerfile = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "apptainer_def") {
            row.apptainer_def = Some(value);
        } else if let Some(value) = parse_toml_string(trimmed, "version_cmd") {
            row.version_cmd = Some(value);
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
    let parsed: toml::Value = raw.parse().context("parse registry toml")?;
    let mut tools = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|tool| {
            tool.get("id")
                .and_then(toml::Value::as_str)
                .map(str::to_string)
        })
        .collect::<Vec<_>>();
    tools.sort();
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
    let parsed: toml::Value = raw.parse().context("parse registry toml")?;
    let mut stages = parsed
        .get("stages")
        .and_then(toml::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|stage| {
            stage
                .get("id")
                .and_then(toml::Value::as_str)
                .map(str::to_string)
        })
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
    let parsed: toml::Value = raw.parse().context("parse registry toml")?;
    if let Some(tool) = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .and_then(|arr| {
            arr.iter()
                .find(|tool| tool.get("id").and_then(toml::Value::as_str) == Some(id))
        })
    {
        crate::commands::cli::render::json::print_pretty(tool)?;
        return Ok(());
    }
    if let Some(stage) = parsed
        .get("stages")
        .and_then(toml::Value::as_array)
        .and_then(|arr| {
            arr.iter()
                .find(|stage| stage.get("id").and_then(toml::Value::as_str) == Some(id))
        })
    {
        crate::commands::cli::render::json::print_pretty(stage)?;
        return Ok(());
    }
    Err(anyhow!("registry id not found: {id}"))
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
