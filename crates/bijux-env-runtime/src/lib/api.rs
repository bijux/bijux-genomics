use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RunnerKind {
    Docker,
    Singularity,
    Apptainer,
}

impl fmt::Display for RunnerKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            RunnerKind::Docker => "docker",
            RunnerKind::Singularity => "singularity",
            RunnerKind::Apptainer => "apptainer",
        };
        write!(f, "{value}")
    }
}

impl FromStr for RunnerKind {
    type Err = EnvError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "docker" => Ok(RunnerKind::Docker),
            "singularity" => Ok(RunnerKind::Singularity),
            "apptainer" => Ok(RunnerKind::Apptainer),
            other => Err(EnvError::Parse(format!("unknown runner kind: {other}"))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlatformSpec {
    pub name: String,
    pub runner: RunnerKind,
    pub container_dir: PathBuf,
    pub image_prefix: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PlatformSpecRaw {
    pub runner: RunnerKind,
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
    pub runner: RunnerKind,
}

impl ResolvedImage {
    #[must_use]
    pub fn is_compatible(&self, runner: RunnerKind) -> bool {
        match runner {
            RunnerKind::Docker => self.runner == RunnerKind::Docker,
            RunnerKind::Apptainer | RunnerKind::Singularity => {
                matches!(self.runner, RunnerKind::Apptainer | RunnerKind::Singularity)
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum EnvError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("parse error: {0}")]
    Parse(String),
    #[error("platform error: {0}")]
    Platform(String),
    #[error("runner unavailable")]
    RunnerUnavailable,
    #[error("dockerfile error: {0}")]
    Dockerfile(String),
    #[error("image error: {0}")]
    Image(String),
}

/// Load platforms from configs/platforms.yaml and resolve the selected platform.
///
/// # Errors
/// Returns an error if the config file cannot be read or parsed, or if the platform is missing.
pub fn load_platform(name: Option<&str>) -> Result<PlatformSpec, EnvError> {
    let path = Path::new("configs").join("platforms.yaml");
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
    let file: PlatformsFile = serde_yaml::from_str(&contents)?;
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

/// List available runners based on local command probes.
///
/// # Errors
/// Returns an error if probing cannot be performed.
pub fn available_runners() -> Result<Vec<RunnerKind>, EnvError> {
    Ok(available_runners_with(probe_command))
}

pub(crate) fn available_runners_with<F>(probe: F) -> Vec<RunnerKind>
where
    F: Fn(&str) -> bool,
{
    let mut runners = Vec::new();
    if probe("docker") {
        runners.push(RunnerKind::Docker);
    }
    if probe("apptainer") {
        runners.push(RunnerKind::Apptainer);
    }
    if probe("singularity") {
        runners.push(RunnerKind::Singularity);
    }
    runners
}

fn probe_command(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

/// Select the best runner with a fallback order.
///
/// # Errors
/// Returns an error if no runners are available.
pub fn select_best_runner(
    preferred: RunnerKind,
    available: &[RunnerKind],
) -> Result<RunnerKind, EnvError> {
    if available.contains(&preferred) {
        return Ok(preferred);
    }
    if available.contains(&RunnerKind::Apptainer) {
        return Ok(RunnerKind::Apptainer);
    }
    if available.contains(&RunnerKind::Singularity) {
        return Ok(RunnerKind::Singularity);
    }
    if available.contains(&RunnerKind::Docker) {
        return Ok(RunnerKind::Docker);
    }
    Err(EnvError::RunnerUnavailable)
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

/// Load tool images from configs/images.yaml.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or contains invalid entries.
pub fn load_image_catalog() -> Result<HashMap<String, ToolImageSpec>, EnvError> {
    let path = Path::new("configs").join("images.yaml");
    load_image_catalog_from_file(&path)
}

/// Load tool images from a specific YAML file.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or contains invalid entries.
pub(crate) fn load_image_catalog_from_file(
    path: &Path,
) -> Result<HashMap<String, ToolImageSpec>, EnvError> {
    let contents = std::fs::read_to_string(path)?;
    let raw: HashMap<String, ToolImageSpec> = serde_yaml::from_str(&contents)?;
    let mut catalog = HashMap::new();
    for (key, mut spec) in raw {
        if key.trim().is_empty() {
            return Err(EnvError::Image(
                "empty tool name in images.yaml".to_string(),
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

#[must_use]
pub fn cache_dir(runner: RunnerKind) -> PathBuf {
    let home = std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from);
    match runner {
        RunnerKind::Docker => home
            .join(".cache")
            .join("bijux")
            .join("docker")
            .join("images"),
        RunnerKind::Apptainer | RunnerKind::Singularity => home
            .join(".cache")
            .join("bijux")
            .join("apptainer")
            .join("sif"),
    }
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
        std::fs::create_dir_all(&self.root)?;
        let digest = hash_file_sha256(fasta)?;
        let ref_root = self.root.join(&digest);
        std::fs::create_dir_all(&ref_root)?;
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
            run_command("samtools", &["faidx", fasta_target.to_str().unwrap_or("")])?;
        }
        if request.build_dict && !dict.exists() {
            run_command(
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
            run_command("bwa", &["index", fasta_target.to_str().unwrap_or("")])?;
        }
        if request.build_bowtie2_index
            && !bowtie2_prefix.with_extension("1.bt2").exists()
        {
            run_command(
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

fn run_command(cmd: &str, args: &[&str]) -> Result<(), EnvError> {
    let status = Command::new(cmd).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(EnvError::Platform(format!("command failed: {cmd} {args:?}")))
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

#[must_use]
pub fn docker_image_exists(image: &ResolvedImage) -> bool {
    docker_image_exists_with(image, |args| {
        Command::new("docker")
            .args(args)
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
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
    let cache = cache_dir(RunnerKind::Apptainer);
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
        resolve_image, select_best_runner, PlatformSpec, ResolvedImage, RunnerKind, ToolImageSpec,
    };
}
