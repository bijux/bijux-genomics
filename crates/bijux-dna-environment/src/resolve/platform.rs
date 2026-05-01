use std::path::{Path, PathBuf};

use super::types::{PlatformSpecRaw, PlatformsFile};
use super::{EnvError, PlatformSpec, RuntimeKind};

/// Load platforms from configs/runtime/platforms.toml and resolve the selected platform.
///
/// # Errors
/// Returns an error if the config file cannot be read or parsed, or if the platform is missing.
pub(super) fn load_platform(name: Option<&str>) -> Result<PlatformSpec, EnvError> {
    let path = bijux_dna_infra::configs_file(Path::new("."), "runtime/platforms.toml");
    load_platform_from_file(&path, name)
}

/// Load platforms from a specific path.
///
/// # Errors
/// Returns an error if the config file cannot be read or parsed, or if the platform is missing.
pub(super) fn load_platform_from_file(
    path: &Path,
    name: Option<&str>,
) -> Result<PlatformSpec, EnvError> {
    let contents = std::fs::read_to_string(path)?;
    let file: PlatformsFile = bijux_dna_infra::formats::parse_toml(&contents)
        .map_err(|err| EnvError::Parse(err.message))?;
    let selected = name.unwrap_or(&file.default);
    let raw = file
        .platforms
        .get(selected)
        .ok_or_else(|| EnvError::Platform(format!("unknown platform: {selected}")))?;

    Ok(PlatformSpec {
        name: selected.to_string(),
        runner: raw.runner,
        container_dir: resolved_container_dir(raw),
        image_prefix: raw.image_prefix.clone(),
        arch: raw.arch.clone(),
    })
}

fn resolved_container_dir(raw: &PlatformSpecRaw) -> PathBuf {
    if !matches!(raw.runner, RuntimeKind::Apptainer | RuntimeKind::Singularity) {
        return raw.container_dir.clone();
    }
    if let Ok(path) = std::env::var("BIJUX_APPTAINER_CONTAINER_DIR") {
        if !path.trim().is_empty() {
            return PathBuf::from(path);
        }
    }
    if let Ok(path) = std::env::var("BIJUX_CACHE_ROOT") {
        if !path.trim().is_empty() {
            return PathBuf::from(path).join("bijux-dna-container").join("apptainer").join("sif");
        }
    }
    if let Ok(path) = std::env::var("BIJUX_HPC_ROOT") {
        if !path.trim().is_empty() {
            return PathBuf::from(path)
                .join(".cache")
                .join("bijux-dna-container")
                .join("apptainer")
                .join("sif");
        }
    }
    raw.container_dir.clone()
}

pub(super) fn available_runners_with<F>(probe: F) -> Vec<RuntimeKind>
where
    F: Fn(&str) -> bool,
{
    let mut runners = Vec::new();
    if probe("sh") {
        runners.push(RuntimeKind::Local);
    }
    if probe("docker") {
        runners.push(RuntimeKind::Docker);
    }
    if probe("apptainer") {
        runners.push(RuntimeKind::Apptainer);
    }
    if probe("singularity") {
        runners.push(RuntimeKind::Singularity);
    }
    runners
}

/// Select the best runner with a fallback order.
///
/// # Errors
/// Returns an error if no runners are available.
pub(super) fn select_best_runner(
    preferred: RuntimeKind,
    available: &[RuntimeKind],
) -> Result<RuntimeKind, EnvError> {
    if available.contains(&preferred) {
        return Ok(preferred);
    }
    if available.contains(&RuntimeKind::Local) {
        return Ok(RuntimeKind::Local);
    }
    if available.contains(&RuntimeKind::Apptainer) {
        return Ok(RuntimeKind::Apptainer);
    }
    if available.contains(&RuntimeKind::Singularity) {
        return Ok(RuntimeKind::Singularity);
    }
    if available.contains(&RuntimeKind::Docker) {
        return Ok(RuntimeKind::Docker);
    }
    Err(EnvError::RuntimeUnavailable)
}
