//! Deterministic environment resolution and digest pinning.
//!
//! Responsibilities: resolve platform + image catalog into pinned digests.
//! Invariants: same inputs produce identical resolved specs; no network pulls.

use std::collections::HashMap;
#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

mod cache;
mod catalog;
mod commands;
mod platform;
mod reference;
mod smoke;
mod types;

pub use commands::{available_runners, docker_image_exists};
pub use reference::{ReferenceBuildRequest, ReferenceRecord, ReferenceRegistry};
pub use types::{
    EnvError, ImageRef, PlatformSpec, ResolvedImage, RuntimeKind, ToolImageCatalog, ToolImageSpec,
};
/// Resolver entrypoint for environment specs and image catalog.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvironmentResolver;

impl EnvironmentResolver {
    /// # Errors
    /// Returns an error if the platform cannot be loaded.
    pub fn load_platform(name: Option<&str>) -> Result<PlatformSpec, EnvError> {
        load_platform(name)
    }

    /// # Errors
    /// Returns an error if the image catalog cannot be loaded.
    pub fn load_image_catalog() -> Result<HashMap<String, ToolImageSpec>, EnvError> {
        load_image_catalog()
    }

    /// # Errors
    /// Returns an error if the image cannot be resolved.
    pub fn resolve_image(
        tool: &ToolImageSpec,
        platform: &PlatformSpec,
    ) -> Result<ResolvedImage, EnvError> {
        resolve_image(tool, platform)
    }

    /// # Errors
    /// Returns an error if validation fails.
    pub fn validate_images_for_stage(
        catalog: &HashMap<String, ToolImageSpec>,
        tools: &[&str],
    ) -> Result<(), EnvError> {
        validate_images_for_stage(catalog, tools)
    }
}

/// Load platforms from configs/runtime/platforms.toml and resolve the selected platform.
///
/// # Errors
/// Returns an error if the config file cannot be read or parsed, or if the platform is missing.
pub fn load_platform(name: Option<&str>) -> Result<PlatformSpec, EnvError> {
    platform::load_platform(name)
}

/// Load platforms from a specific path.
///
/// # Errors
/// Returns an error if the config file cannot be read or parsed, or if the platform is missing.
#[cfg(test)]
pub(crate) fn load_platform_from_file(
    path: &Path,
    name: Option<&str>,
) -> Result<PlatformSpec, EnvError> {
    platform::load_platform_from_file(path, name)
}

pub(crate) fn available_runners_with<F>(probe: F) -> Vec<RuntimeKind>
where
    F: Fn(&str) -> bool,
{
    platform::available_runners_with(probe)
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

#[cfg(test)]
mod tests {
    use super::{load_platform_from_file, RuntimeKind};

    #[test]
    fn load_platform_prefers_cache_root_for_apptainer_platforms() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-platform-cache-root")?;
        let platform_path = temp.path().join("platforms.toml");
        bijux_dna_infra::write_bytes(
            &platform_path,
            br#"
default = "apptainer-amd64"

[platforms.apptainer-amd64]
runner = "apptainer"
container_dir = "containers/apptainer/sif"
image_prefix = "bijuxdna"
arch = "amd64"
"#,
        )?;
        std::env::set_var("BIJUX_CACHE_ROOT", "/var/tmp/bijux-cache-root");
        let platform = load_platform_from_file(&platform_path, Some("apptainer-amd64"))?;
        std::env::remove_var("BIJUX_CACHE_ROOT");

        assert_eq!(platform.runner, RuntimeKind::Apptainer);
        assert_eq!(
            platform.container_dir,
            std::path::Path::new("/var/tmp/bijux-cache-root")
                .join("bijux-dna-container")
                .join("apptainer")
                .join("sif")
        );
        Ok(())
    }

    #[test]
    fn load_platform_keeps_relative_apptainer_dir_without_cache_env() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-platform-relative-apptainer")?;
        let platform_path = temp.path().join("platforms.toml");
        bijux_dna_infra::write_bytes(
            &platform_path,
            br#"
default = "apptainer-amd64"

[platforms.apptainer-amd64]
runner = "apptainer"
container_dir = "containers/apptainer/sif"
image_prefix = "bijuxdna"
arch = "amd64"
"#,
        )?;
        std::env::remove_var("BIJUX_CACHE_ROOT");
        std::env::remove_var("BIJUX_HPC_ROOT");
        std::env::remove_var("BIJUX_APPTAINER_CONTAINER_DIR");
        let platform = load_platform_from_file(&platform_path, Some("apptainer-amd64"))?;

        assert_eq!(platform.runner, RuntimeKind::Apptainer);
        assert_eq!(
            platform.container_dir,
            std::path::Path::new("containers")
                .join("apptainer")
                .join("sif")
        );
        Ok(())
    }
}

/// Load tool images from configs/ci/tools/images.toml.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or contains invalid entries.
pub fn load_image_catalog() -> Result<HashMap<String, ToolImageSpec>, EnvError> {
    catalog::load_image_catalog()
}

/// Load tool images from a specific TOML file.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or contains invalid entries.
#[cfg(test)]
pub(crate) fn load_image_catalog_from_file(
    path: &Path,
) -> Result<HashMap<String, ToolImageSpec>, EnvError> {
    catalog::load_image_catalog_from_file(path)
}

#[cfg(test)]
pub(crate) fn hydrate_catalog_digests_from_registry(
    catalog: &mut HashMap<String, ToolImageSpec>,
    registry_path: &Path,
) -> Result<(), EnvError> {
    catalog::hydrate_catalog_digests_from_registry(catalog, registry_path)
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
    smoke::run_shell_capture(cmd)
}

#[must_use]
pub fn cache_dir(runner: RuntimeKind) -> PathBuf {
    cache::cache_dir(runner)
}

pub(crate) fn docker_image_exists_with<F>(image: &ResolvedImage, runner: F) -> bool
where
    F: Fn(&[&str]) -> bool,
{
    cache::docker_image_exists_with(image, runner)
}

#[must_use]
pub fn apptainer_sif_path(image: &ResolvedImage) -> PathBuf {
    cache::apptainer_sif_path(image)
}

pub mod api {
    pub use super::{
        available_runners, cache_dir, docker_image_exists, load_image_catalog, load_platform,
        resolve_image, run_smoke_script_batch, select_best_runner, PlatformSpec, ResolvedImage,
        RuntimeKind, ToolImageSpec,
    };
}

#[must_use]
pub fn reference_cache_dir() -> PathBuf {
    cache::reference_cache_dir()
}
