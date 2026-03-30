use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

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
pub(super) struct PlatformSpecRaw {
    pub runner: RuntimeKind,
    pub container_dir: PathBuf,
    pub image_prefix: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PlatformsFile {
    pub default: String,
    pub platforms: BTreeMap<String, PlatformSpecRaw>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(super) struct RegistryImagePinFile {
    #[serde(default)]
    pub tools: Vec<RegistryImagePinRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(super) struct RegistryImagePinRow {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub container_ref: Option<String>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub shipping_policy: Option<String>,
}

pub trait ToolImageCatalog {
    fn get(&self, key: &str) -> Option<&ToolImageSpec>;
}

impl<S: std::hash::BuildHasher> ToolImageCatalog for HashMap<String, ToolImageSpec, S> {
    fn get(&self, key: &str) -> Option<&ToolImageSpec> {
        HashMap::get(self, key)
    }
}

impl ToolImageCatalog for BTreeMap<String, ToolImageSpec> {
    fn get(&self, key: &str) -> Option<&ToolImageSpec> {
        BTreeMap::get(self, key)
    }
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
