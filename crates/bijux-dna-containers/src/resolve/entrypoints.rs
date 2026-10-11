use std::collections::HashMap;
use std::path::PathBuf;

use super::{
    cache, catalog, platform, shell, smoke, EnvError, PlatformSpec, ResolvedImage, RuntimeKind,
    ToolImageSpec,
};

/// Load platforms from configs/runtime/platforms.toml and resolve the selected platform.
///
/// # Errors
/// Returns an error if the config file cannot be read or parsed, or if the platform is missing.
pub fn load_platform(name: Option<&str>) -> Result<PlatformSpec, EnvError> {
    platform::load_platform(name)
}

/// Select the best runner with a fallback order.
///
/// # Errors
/// Returns an error if no runners are available.
pub fn select_best_runner(
    preferred: RuntimeKind,
    available: &[RuntimeKind],
) -> Result<RuntimeKind, EnvError> {
    platform::select_best_runner(preferred, available)
}

/// Resolve an image reference for a tool and platform.
///
/// # Errors
/// Returns an error if the tool name violates image naming rules.
pub fn resolve_image(
    tool: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage, EnvError> {
    catalog::resolve_image(tool, platform)
}

/// Load tool images from configs/ci/tools/images.toml.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or contains invalid entries.
pub fn load_image_catalog() -> Result<HashMap<String, ToolImageSpec>, EnvError> {
    catalog::load_image_catalog()
}

/// Validate that tools have image entries.
///
/// # Errors
/// Returns an error if any tool is missing from the catalog.
#[allow(clippy::implicit_hasher)]
pub fn validate_images_for_stage(
    catalog: &HashMap<String, ToolImageSpec>,
    tools: &[&str],
) -> Result<(), EnvError> {
    catalog::validate_images_for_stage(catalog, tools)
}

/// Execute container smoke contract script for a runtime/tool pair.
///
/// # Errors
/// Returns an error when the runtime is unsupported or smoke script exits non-zero.
pub fn run_smoke_script(runtime: &str, tool: &str) -> anyhow::Result<()> {
    smoke::run_smoke_script(runtime, tool)
}

/// Execute smoke contract script for a runtime with multiple tools.
///
/// # Errors
/// Returns an error when runtime is unsupported or smoke script exits non-zero.
pub fn run_smoke_script_batch(
    runtime: &str,
    tools: &[String],
    smoke_level: &str,
) -> anyhow::Result<()> {
    smoke::run_smoke_script_batch(runtime, tools, smoke_level)
}

/// Execute a shell command and capture stdout/stderr.
///
/// # Errors
/// Returns an error when command execution fails or exits non-zero.
pub fn run_shell_capture(cmd: &str) -> anyhow::Result<String> {
    shell::run_shell_capture(cmd)
}

#[must_use]
pub fn cache_dir(runner: RuntimeKind) -> PathBuf {
    cache::cache_dir(runner)
}

#[must_use]
pub fn apptainer_sif_path(image: &ResolvedImage) -> PathBuf {
    cache::apptainer_sif_path(image)
}
