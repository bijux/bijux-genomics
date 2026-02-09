use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

use anyhow::{anyhow, Context, Result};
use bijux_dna_api::v1::api::env::{
    available_runners, cache_dir, docker_image_exists, resolve_image, PlatformSpec, RuntimeKind,
    ToolImageSpec,
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
    let parsed: toml::Value = raw.parse().context("parse tools registry TOML")?;
    let tools = parsed
        .get("tools")
        .and_then(toml::Value::as_array)
        .ok_or_else(|| anyhow!("missing [[tools]] in {}", registry_path.display()))?;

    println!("tool\thas_docker\thas_apptainer\thas_smoke\tpinned");
    for entry in tools {
        let id = entry
            .get("id")
            .and_then(toml::Value::as_str)
            .unwrap_or("<missing>");
        let runtimes = entry
            .get("runtimes")
            .and_then(toml::Value::as_array)
            .cloned()
            .unwrap_or_default();
        let has_docker = runtimes
            .iter()
            .any(|v| v.as_str().map(|s| s == "docker").unwrap_or(false))
            && entry
                .get("dockerfile")
                .and_then(toml::Value::as_str)
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
        let has_apptainer = runtimes
            .iter()
            .any(|v| v.as_str().map(|s| s == "apptainer").unwrap_or(false))
            && entry
                .get("apptainer_def")
                .and_then(toml::Value::as_str)
                .map(|s| !s.trim().is_empty())
                .unwrap_or(false);
        let has_smoke = entry
            .get("version_cmd")
            .and_then(toml::Value::as_str)
            .map(|s| !s.trim().is_empty())
            .unwrap_or(false);
        let pinned = entry
            .get("pinned_commit")
            .and_then(toml::Value::as_str)
            .map(|s| {
                let t = s.trim();
                t.len() == 40 && t.chars().all(|c| c.is_ascii_hexdigit())
            })
            .unwrap_or(false);
        println!("{id}\t{has_docker}\t{has_apptainer}\t{has_smoke}\t{pinned}");
    }
    Ok(())
}

/// # Errors
/// Returns an error if smoke script execution fails.
pub fn run_env_smoke(runtime: &str, tool: &str) -> Result<()> {
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

    let status = Command::new("sh")
        .arg(script)
        .env("TOOLS", tool)
        .env("JOBS", "1")
        .env("SMOKE_LEVEL", "contract")
        .status()
        .with_context(|| format!("run smoke script {script}"))?;
    if !status.success() {
        return Err(anyhow!(
            "smoke failed for runtime={runtime} tool={tool} (exit={status})"
        ));
    }
    Ok(())
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
