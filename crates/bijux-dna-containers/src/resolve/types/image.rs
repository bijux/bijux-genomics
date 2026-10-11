use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use super::{EnvError, RuntimeKind};

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
            RuntimeKind::Local => self.runner == RuntimeKind::Local,
            RuntimeKind::Docker => self.runner == RuntimeKind::Docker,
            RuntimeKind::Apptainer | RuntimeKind::Singularity => {
                matches!(self.runner, RuntimeKind::Apptainer | RuntimeKind::Singularity)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(in crate::resolve) struct RegistryImagePinFile {
    #[serde(default)]
    pub tools: Vec<RegistryImagePinRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(in crate::resolve) struct RegistryImagePinRow {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub container_ref: Option<String>,
}

impl std::str::FromStr for RuntimeKind {
    type Err = EnvError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "local" => Ok(RuntimeKind::Local),
            "docker" => Ok(RuntimeKind::Docker),
            "singularity" => Ok(RuntimeKind::Singularity),
            "apptainer" => Ok(RuntimeKind::Apptainer),
            other => Err(EnvError::Parse(format!("unknown runner kind: {other}"))),
        }
    }
}
