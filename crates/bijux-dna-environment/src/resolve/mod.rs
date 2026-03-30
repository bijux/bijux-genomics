//! Deterministic environment resolution and digest pinning.
//!
//! Responsibilities: resolve platform + image catalog into pinned digests.
//! Invariants: same inputs produce identical resolved specs; no network pulls.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

mod catalog;
mod commands;
mod platform;
mod smoke;
mod types;

pub use commands::{available_runners, docker_image_exists};
pub use types::{
    EnvError, ImageRef, PlatformSpec, ResolvedImage, RuntimeKind, ToolImageCatalog,
    ToolImageSpec,
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
        std::fs::write(
            &platform_path,
            r#"
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
        std::fs::write(
            &platform_path,
            r#"
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
pub(crate) fn load_image_catalog_from_file(
    path: &Path,
) -> Result<HashMap<String, ToolImageSpec>, EnvError> {
    catalog::load_image_catalog_from_file(path)
}

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
    let cache_root = base_cache_root();
    match runner {
        RuntimeKind::Docker => cache_root.join("bijux").join("docker").join("images"),
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            cache_root.join("bijux").join("apptainer").join("sif")
        }
    }
}

fn base_cache_root() -> PathBuf {
    std::env::var_os("BIJUX_CACHE_ROOT")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("XDG_CACHE_HOME")
                .filter(|value| !value.is_empty())
                .map(PathBuf::from)
        })
        .unwrap_or_else(|| {
            std::env::var_os("HOME")
                .map_or_else(|| PathBuf::from("."), PathBuf::from)
                .join(".cache")
        })
}

pub(crate) fn docker_image_exists_with<F>(image: &ResolvedImage, runner: F) -> bool
where
    F: Fn(&[&str]) -> bool,
{
    runner(&["image", "inspect", &image.full_name])
}

#[must_use]
pub fn apptainer_sif_path(image: &ResolvedImage) -> PathBuf {
    let cache = cache_dir(RuntimeKind::Apptainer);
    let tool = extract_tool_name(&image.full_name);
    let version_or_digest = extract_version_or_digest(&image.full_name, &image.arch);
    cache.join(format!("{}-{}-{}.sif", tool, version_or_digest, image.arch))
}

pub(crate) fn extract_tool_name(full_name: &str) -> String {
    let without_prefix = full_name.rsplit_once('/').map_or(full_name, |(_, t)| t);
    let tool = without_prefix
        .split(['@', ':'])
        .next()
        .unwrap_or(without_prefix);
    tool.to_string()
}

pub(crate) fn extract_version_or_digest(full_name: &str, arch: &str) -> String {
    if let Some((_, digest)) = full_name.rsplit_once('@') {
        digest.to_string()
    } else if let Some((_, tag)) = full_name.rsplit_once(':') {
        tag.strip_suffix(&format!("-{arch}"))
            .unwrap_or(tag)
            .to_string()
    } else {
        "unknown".to_string()
    }
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
    base_cache_root().join("bijux").join("references")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceRecord {
    pub digest: String,
    pub root: PathBuf,
    pub fasta: PathBuf,
    pub fai: PathBuf,
    pub dict: PathBuf,
    pub bwa_prefix: PathBuf,
    pub bowtie2_prefix: PathBuf,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct ReferenceBuildRequest {
    pub build_fai: bool,
    pub build_dict: bool,
    pub build_bwa_index: bool,
    pub build_bowtie2_index: bool,
}

#[derive(Debug, Clone)]
pub struct ReferenceRegistry {
    root: PathBuf,
}

impl ReferenceRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self {
            root: reference_cache_dir(),
        }
    }

    /// # Errors
    /// Returns an error if the reference cannot be registered or prepared.
    pub fn prepare_reference(
        &self,
        fasta: &Path,
        request: &ReferenceBuildRequest,
    ) -> Result<ReferenceRecord, EnvError> {
        bijux_dna_infra::ensure_dir(&self.root)?;
        let digest = hash_file_sha256(fasta)?;
        let ref_root = self.root.join(&digest);
        bijux_dna_infra::ensure_dir(&ref_root)?;
        let fasta_target = ref_root.join(
            fasta
                .file_name()
                .ok_or_else(|| EnvError::Parse("invalid reference path".to_string()))?,
        );
        if !fasta_target.exists() {
            std::fs::copy(fasta, &fasta_target)?;
        }
        let fai = fasta_target.with_extension("fai");
        let dict = fasta_target.with_extension("dict");
        let bwa_prefix = fasta_target.clone();
        let bowtie2_prefix = fasta_target.clone();

        if request.build_fai && !fai.exists() {
            commands::run_command("samtools", &["faidx", fasta_target.to_str().unwrap_or("")])?;
        }
        if request.build_dict && !dict.exists() {
            commands::run_command(
                "gatk",
                &[
                    "CreateSequenceDictionary",
                    "-R",
                    fasta_target.to_str().unwrap_or(""),
                    "-O",
                    dict.to_str().unwrap_or(""),
                ],
            )?;
        }
        if request.build_bwa_index && !bwa_prefix.with_extension("bwt").exists() {
            commands::run_command("bwa", &["index", fasta_target.to_str().unwrap_or("")])?;
        }
        if request.build_bowtie2_index && !bowtie2_prefix.with_extension("1.bt2").exists() {
            commands::run_command(
                "bowtie2-build",
                &[
                    fasta_target.to_str().unwrap_or(""),
                    bowtie2_prefix.to_str().unwrap_or(""),
                ],
            )?;
        }

        Ok(ReferenceRecord {
            digest,
            root: ref_root,
            fasta: fasta_target,
            fai,
            dict,
            bwa_prefix,
            bowtie2_prefix,
        })
    }
}

impl Default for ReferenceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

fn hash_file_sha256(path: &Path) -> Result<String, EnvError> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = std::io::Read::read(&mut file, &mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}
