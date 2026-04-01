use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::RuntimeKind;

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
pub(in crate::resolve) struct PlatformSpecRaw {
    pub runner: RuntimeKind,
    pub container_dir: PathBuf,
    pub image_prefix: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::resolve) struct PlatformsFile {
    pub default: String,
    pub platforms: BTreeMap<String, PlatformSpecRaw>,
}
