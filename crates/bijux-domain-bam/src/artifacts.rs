//! Canonical BAM artifacts and references.

use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BamPath(pub PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BaiPath(pub PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReferenceFasta(pub PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct FaiPath(pub PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DictPath(pub PathBuf);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BedRegions(pub PathBuf);

impl BamPath {
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}

impl BaiPath {
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}

impl ReferenceFasta {
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}

impl FaiPath {
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}

impl DictPath {
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}

impl BedRegions {
    #[must_use]
    pub fn as_path(&self) -> &Path {
        self.0.as_path()
    }
}
