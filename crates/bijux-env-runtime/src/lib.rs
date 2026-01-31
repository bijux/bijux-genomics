use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runner_kind_from_str() -> Result<(), EnvError> {
        assert_eq!(RunnerKind::from_str("docker")?, RunnerKind::Docker);
        assert_eq!(
            RunnerKind::from_str("singularity")?,
            RunnerKind::Singularity
        );
        assert_eq!(RunnerKind::from_str("apptainer")?, RunnerKind::Apptainer);
        Ok(())
    }

    #[test]
    fn platform_spec_yaml_roundtrip() -> Result<(), EnvError> {
        let yaml = r"
name: docker-mac-arm64
runner: docker
container_dir: containers/docker/arm64
image_prefix: bijuxdna
arch: arm64
";
        let spec: PlatformSpec = serde_yaml::from_str(yaml)?;
        assert_eq!(spec.name, "docker-mac-arm64");
        assert_eq!(spec.runner, RunnerKind::Docker);
        let out = serde_yaml::to_string(&spec)?;
        let spec2: PlatformSpec = serde_yaml::from_str(&out)?;
        assert_eq!(spec2.name, spec.name);
        Ok(())
    }

    #[test]
    fn image_ref_formats() {
        let image = ImageRef {
            tool: "fastp".to_string(),
            version: "0.23.4".to_string(),
            arch: "arm64".to_string(),
        };
        assert_eq!(
            image.to_full_name("bijuxdna"),
            "bijuxdna/fastp:0.23.4-arm64"
        );
    }

    #[test]
    fn available_runners_mocked() {
        let runners = available_runners_with(|cmd| cmd == "docker");
        assert_eq!(runners, vec![RunnerKind::Docker]);
    }

    #[test]
    fn select_best_runner_prefers_available() -> Result<(), EnvError> {
        let available = vec![RunnerKind::Apptainer, RunnerKind::Docker];
        let selected = select_best_runner(RunnerKind::Docker, &available)?;
        assert_eq!(selected, RunnerKind::Docker);
        Ok(())
    }

    #[test]
    fn select_best_runner_fallbacks() -> Result<(), EnvError> {
        let available = vec![RunnerKind::Singularity];
        let selected = select_best_runner(RunnerKind::Docker, &available)?;
        assert_eq!(selected, RunnerKind::Singularity);
        Ok(())
    }

    #[test]
    fn select_best_runner_errors() {
        let available = Vec::new();
        assert!(select_best_runner(RunnerKind::Docker, &available).is_err());
    }

    #[test]
    fn tool_image_spec_constructs() {
        let spec = ToolImageSpec {
            tool: "fastp".to_string(),
            version: "0.23.4".to_string(),
            digest: None,
        };
        assert_eq!(spec.tool, "fastp");
    }

    #[test]
    fn resolve_image_builds_full_name() -> Result<(), EnvError> {
        let platform = PlatformSpec {
            name: "docker-mac-arm64".to_string(),
            runner: RunnerKind::Docker,
            container_dir: PathBuf::from("containers/docker/arm64"),
            image_prefix: "bijuxdna".to_string(),
            arch: "arm64".to_string(),
        };
        let tool = ToolImageSpec {
            tool: "fastp".to_string(),
            version: "0.23.4".to_string(),
            digest: None,
        };
        let resolved = resolve_image(&tool, &platform)?;
        assert_eq!(resolved.full_name, "bijuxdna/fastp:0.23.4-arm64");
        assert_eq!(resolved.arch, "arm64");
        assert_eq!(resolved.runner, RunnerKind::Docker);
        Ok(())
    }

    #[test]
    fn resolved_image_compatibility() {
        let image = ResolvedImage {
            full_name: "bijuxdna/fastp:0.23.4-arm64".to_string(),
            arch: "arm64".to_string(),
            runner: RunnerKind::Docker,
        };
        assert!(image.is_compatible(RunnerKind::Docker));
        assert!(!image.is_compatible(RunnerKind::Apptainer));
        assert!(!image.is_compatible(RunnerKind::Singularity));

        let oci = ResolvedImage {
            full_name: "bijuxdna/fastp:0.23.4-arm64".to_string(),
            arch: "arm64".to_string(),
            runner: RunnerKind::Apptainer,
        };
        assert!(oci.is_compatible(RunnerKind::Apptainer));
        assert!(oci.is_compatible(RunnerKind::Singularity));
        assert!(!oci.is_compatible(RunnerKind::Docker));
    }

    #[test]
    fn load_image_catalog_parses() -> Result<(), EnvError> {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("bijux_images.yaml");
        std::fs::write(
            &path,
            "fastp:\n  version: \"0.23.4\"\n  digest: \"sha256:abc123\"\n\nbwa:\n  version: \"0.7.17\"\n",
        )?;
        let catalog = load_image_catalog_from_file(&path)?;
        assert!(catalog.contains_key("fastp"));
        let _ = std::fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn validate_images_for_stage_errors() {
        let mut catalog = HashMap::new();
        catalog.insert(
            "fastp".to_string(),
            ToolImageSpec {
                tool: "fastp".to_string(),
                version: "0.23.4".to_string(),
                digest: None,
            },
        );
        match validate_images_for_stage(&catalog, &["fastp", "bwa"]) {
            Ok(()) => panic!("expected error for missing bwa"),
            Err(err) => {
                assert!(format!("{err}").contains("bwa"));
            }
        }
    }

    #[test]
    fn resolve_image_with_digest() -> Result<(), EnvError> {
        let platform = PlatformSpec {
            name: "docker-mac-arm64".to_string(),
            runner: RunnerKind::Docker,
            container_dir: PathBuf::from("containers/docker/arm64"),
            image_prefix: "bijuxdna".to_string(),
            arch: "arm64".to_string(),
        };
        let tool = ToolImageSpec {
            tool: "fastp".to_string(),
            version: "0.23.4".to_string(),
            digest: Some("sha256:abc123".to_string()),
        };
        let resolved = resolve_image(&tool, &platform)?;
        assert_eq!(resolved.full_name, "bijuxdna/fastp@sha256:abc123");
        Ok(())
    }

    #[test]
    fn cache_dir_is_deterministic() -> Result<(), EnvError> {
        let home = std::env::temp_dir().join("bijux_home");
        std::fs::create_dir_all(&home)?;
        let original = std::env::var_os("HOME");
        std::env::set_var("HOME", &home);
        let docker = cache_dir(RunnerKind::Docker);
        let apptainer = cache_dir(RunnerKind::Apptainer);
        if let Some(value) = original {
            std::env::set_var("HOME", value);
        }
        assert!(docker
            .to_string_lossy()
            .contains(".cache/bijux/docker/images"));
        assert!(apptainer
            .to_string_lossy()
            .contains(".cache/bijux/apptainer/sif"));
        Ok(())
    }

    #[test]
    fn docker_image_exists_mocked() {
        let image = ResolvedImage {
            full_name: "bijuxdna/fastp:0.23.4-arm64".to_string(),
            arch: "arm64".to_string(),
            runner: RunnerKind::Docker,
        };
        let exists = super::docker_image_exists_with(&image, |args| {
            args == ["image", "inspect", "bijuxdna/fastp:0.23.4-arm64"]
        });
        assert!(exists);
    }

    #[test]
    fn apptainer_sif_path_is_deterministic() {
        let image = ResolvedImage {
            full_name: "bijuxdna/fastp@sha256:abc123".to_string(),
            arch: "arm64".to_string(),
            runner: RunnerKind::Apptainer,
        };
        let path = apptainer_sif_path(&image);
        assert!(path
            .to_string_lossy()
            .contains("fastp-sha256:abc123-arm64.sif"));
    }
}
