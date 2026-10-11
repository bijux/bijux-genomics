use serde::{Deserialize, Serialize};

use super::{CatalogCompatibility, MapCompatibility};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CatalogFileDerivation {
    pub source_file_name: String,
    pub tool_id: String,
    pub tool_version: String,
    pub tool_artifact_sha256: String,
    pub argv: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PanelCatalogEntry {
    pub id: String,
    pub species_id: String,
    pub build_id: String,
    #[serde(default)]
    pub status: String,
    pub version: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub lock_ref: String,
    #[serde(default)]
    pub citation: Option<String>,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
    pub compatibility: CatalogCompatibility,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapCatalogEntry {
    pub id: String,
    pub species_id: String,
    pub build_id: String,
    #[serde(default)]
    pub status: String,
    pub version: String,
    #[serde(default)]
    pub lock_ref: String,
    #[serde(default)]
    pub citation: Option<String>,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
    pub compatibility: MapCompatibility,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CatalogFileEntry {
    pub name: String,
    pub path: String,
    pub format: String,
    pub url: String,
    pub checksum_sha256: String,
    #[serde(default)]
    pub size_bytes: u64,
    #[serde(default)]
    pub source_checksum_algorithm: String,
    #[serde(default)]
    pub source_checksum: String,
    #[serde(default)]
    pub source_authority_url: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub derivation: Option<CatalogFileDerivation>,
}
