//! Deterministic environment resolution and digest pinning.
//!
//! Responsibilities: resolve platform + image catalog into pinned digests.
//! Invariants: same inputs produce identical resolved specs; no network pulls.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::Context;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

mod commands;

pub use commands::{available_runners, docker_image_exists};
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeKind {
    Docker,
    Singularity,
    Apptainer,
}

impl fmt::Display for RuntimeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            RuntimeKind::Docker => "docker",
            RuntimeKind::Singularity => "singularity",
            RuntimeKind::Apptainer => "apptainer",
        };
        write!(f, "{value}")
    }
}

impl FromStr for RuntimeKind {
    type Err = EnvError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "docker" => Ok(RuntimeKind::Docker),
            "singularity" => Ok(RuntimeKind::Singularity),
            "apptainer" => Ok(RuntimeKind::Apptainer),
            other => Err(EnvError::Parse(format!("unknown runner kind: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformSpec {
    pub name: String,
    pub runner: RuntimeKind,
    pub container_dir: PathBuf,
    pub image_prefix: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlatformSpecRaw {
    pub runner: RuntimeKind,
    pub container_dir: PathBuf,
    pub image_prefix: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlatformsFile {
    pub default: String,
    pub platforms: BTreeMap<String, PlatformSpecRaw>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageRef {
    pub tool: String,
    pub version: String,
    pub arch: String,
}

impl ImageRef {
    #[must_use]
    pub fn to_full_name(&self, prefix: &str) -> String {
        format!("{}/{}:{}-{}", prefix, self.tool, self.version, self.arch)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolImageSpec {
    #[serde(default)]
    pub tool: String,
    pub version: String,
    #[serde(default)]
    pub digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedImage {
    pub full_name: String,
    pub arch: String,
    pub runner: RuntimeKind,
}

impl ResolvedImage {
    #[must_use]
    pub fn is_compatible(&self, runner: RuntimeKind) -> bool {
        match runner {
            RuntimeKind::Docker => self.runner == RuntimeKind::Docker,
            RuntimeKind::Apptainer | RuntimeKind::Singularity => {
                matches!(
                    self.runner,
                    RuntimeKind::Apptainer | RuntimeKind::Singularity
                )
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("platform error: {0}")]
    Platform(String),
    #[error("runner unavailable")]
    RuntimeUnavailable,
    #[error("dockerfile error: {0}")]
    Dockerfile(String),
    #[error("image error: {0}")]
    Image(String),
}

impl From<bijux_dna_infra::IoError> for EnvError {
    fn from(err: bijux_dna_infra::IoError) -> Self {
        Self::Io(std::io::Error::other(err))
    }
}

/// Load platforms from configs/runtime/platforms.toml and resolve the selected platform.
///
/// # Errors
/// Returns an error if the config file cannot be read or parsed, or if the platform is missing.
pub fn load_platform(name: Option<&str>) -> Result<PlatformSpec, EnvError> {
    let path = bijux_dna_infra::configs_file(Path::new("."), "runtime/platforms.toml");
    load_platform_from_file(&path, name)
}

/// Load platforms from a specific path.
///
/// # Errors
/// Returns an error if the config file cannot be read or parsed, or if the platform is missing.
pub(crate) fn load_platform_from_file(
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
        container_dir: raw.container_dir.clone(),
        image_prefix: raw.image_prefix.clone(),
        arch: raw.arch.clone(),
    })
}

pub(crate) fn available_runners_with<F>(probe: F) -> Vec<RuntimeKind>
where
    F: Fn(&str) -> bool,
{
    let mut runners = Vec::new();
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
pub fn select_best_runner(
    preferred: RuntimeKind,
    available: &[RuntimeKind],
) -> Result<RuntimeKind, EnvError> {
    if available.contains(&preferred) {
        return Ok(preferred);
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

/// Resolve an image reference for a tool and platform.
///
/// # Errors
/// Returns an error if the tool name violates image naming rules.
pub fn resolve_image(
    tool: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage, EnvError> {
    if tool.tool.to_lowercase().contains("base") {
        return Err(EnvError::Image(format!(
            "tool image name must not reference base: {}",
            tool.tool
        )));
    }
    let full_name = if let Some(digest) = tool.digest.as_ref() {
        format!("{}/{}@{}", platform.image_prefix, tool.tool, digest)
    } else {
        format!(
            "{}/{}:{}-{}",
            platform.image_prefix, tool.tool, tool.version, platform.arch
        )
    };
    Ok(ResolvedImage {
        full_name,
        arch: platform.arch.clone(),
        runner: platform.runner,
    })
}

/// Load tool images from configs/ci/tools/images.toml.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or contains invalid entries.
pub fn load_image_catalog() -> Result<HashMap<String, ToolImageSpec>, EnvError> {
    let path = bijux_dna_infra::configs_file(Path::new("."), "ci/tools/images.toml");
    load_image_catalog_from_file(&path)
}

/// Load tool images from a specific TOML file.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or contains invalid entries.
pub(crate) fn load_image_catalog_from_file(
    path: &Path,
) -> Result<HashMap<String, ToolImageSpec>, EnvError> {
    let contents = std::fs::read_to_string(path)?;
    let raw: HashMap<String, ToolImageSpec> = bijux_dna_infra::formats::parse_toml(&contents)
        .map_err(|err| EnvError::Parse(err.message))?;
    let mut catalog = HashMap::new();
    for (key, mut spec) in raw {
        if key.trim().is_empty() {
            return Err(EnvError::Image(
                "empty tool name in images.toml".to_string(),
            ));
        }
        if spec.version.trim().is_empty() {
            return Err(EnvError::Image(format!("empty version for tool {key}")));
        }
        if spec.tool.trim().is_empty() {
            spec.tool.clone_from(&key);
        }
        if catalog.insert(key.clone(), spec).is_some() {
            return Err(EnvError::Image(format!("duplicate tool {key}")));
        }
    }
    Ok(catalog)
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
    for tool in tools {
        if !catalog.contains_key(*tool) {
            return Err(EnvError::Image(format!(
                "missing image entry for tool {tool}"
            )));
        }
    }
    Ok(())
}

/// Execute container smoke contract script for a runtime/tool pair.
///
/// # Errors
/// Returns an error when the runtime is unsupported or smoke script exits non-zero.
pub fn run_smoke_script(runtime: &str, tool: &str) -> anyhow::Result<()> {
    let script = match runtime {
        "docker-arm64" => "scripts/containers/smoke-docker-arm64.sh",
        "docker-amd64" => "scripts/containers/smoke-docker-amd64.sh",
        "apptainer" => "scripts/containers/smoke-apptainer.sh",
        other => {
            anyhow::bail!(
                "unsupported runtime `{other}`; expected docker-arm64 | docker-amd64 | apptainer"
            );
        }
    };
    let status = std::process::Command::new("sh")
        .arg(script)
        .env("TOOLS", tool)
        .env("JOBS", "1")
        .env("SMOKE_LEVEL", "contract")
        .status()?;
    if !status.success() {
        anyhow::bail!("smoke failed for runtime={runtime} tool={tool} (exit={status})");
    }
    Ok(())
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
    let script = match runtime {
        "docker-arm64" => "scripts/containers/smoke-docker-arm64.sh",
        "docker-amd64" => "scripts/containers/smoke-docker-amd64.sh",
        "apptainer" => "scripts/containers/smoke-apptainer.sh",
        other => {
            anyhow::bail!(
                "unsupported runtime `{other}`; expected docker-arm64 | docker-amd64 | apptainer"
            );
        }
    };
    let tools_csv = tools.join(",");
    let status = std::process::Command::new("sh")
        .arg(script)
        .env("TOOLS", tools_csv)
        .env("JOBS", "1")
        .env("SMOKE_LEVEL", smoke_level)
        .status()?;
    if !status.success() {
        anyhow::bail!("smoke failed for runtime={runtime} (exit={status})");
    }
    Ok(())
}

/// Execute a shell command and capture stdout/stderr.
///
/// # Errors
/// Returns an error when command execution fails or exits non-zero.
pub fn run_shell_capture(cmd: &str) -> anyhow::Result<String> {
    if cmd.trim().is_empty() {
        anyhow::bail!("empty command");
    }
    let output = std::process::Command::new("sh")
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
        Err(anyhow::anyhow!("{merged}"))
    }
}

#[must_use]
pub fn cache_dir(runner: RuntimeKind) -> PathBuf {
    let home = std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from);
    match runner {
        RuntimeKind::Docker => home
            .join(".cache")
            .join("bijux")
            .join("docker")
            .join("images"),
        RuntimeKind::Apptainer | RuntimeKind::Singularity => home
            .join(".cache")
            .join("bijux")
            .join("apptainer")
            .join("sif"),
    }
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
    let home = std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from);
    home.join(".cache").join("bijux").join("references")
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
