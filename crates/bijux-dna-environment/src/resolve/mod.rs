//! Deterministic environment resolution and digest pinning.
//!
//! Responsibilities: resolve platform + image catalog into pinned digests.
//! Invariants: same inputs produce identical resolved specs; no network pulls.

#[cfg(test)]
use std::path::Path;
use std::path::PathBuf;

mod cache;
mod catalog;
mod commands;
mod facade;
mod platform;
mod reference;
mod shell;
mod smoke;
mod types;

pub use commands::{available_runners, docker_image_exists};
pub use facade::{
    apptainer_sif_path, cache_dir, load_image_catalog, load_platform, resolve_image,
    run_shell_capture, run_smoke_script, run_smoke_script_batch, select_best_runner,
    validate_images_for_stage, EnvironmentResolver,
};
pub use reference::{ReferenceBuildRequest, ReferenceRecord, ReferenceRegistry};
pub use types::{
    EnvError, ImageRef, PlatformSpec, ResolvedImage, RuntimeKind, ToolImageCatalog, ToolImageSpec,
};

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

pub(crate) fn docker_image_exists_with<F>(image: &ResolvedImage, runner: F) -> bool
where
    F: Fn(&[&str]) -> bool,
{
    cache::docker_image_exists_with(image, runner)
}

#[must_use]
pub fn reference_cache_dir() -> PathBuf {
    cache::reference_cache_dir()
}
