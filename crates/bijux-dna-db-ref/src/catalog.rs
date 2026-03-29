use serde::{Deserialize, Serialize};

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
    pub required: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CatalogCompatibility {
    #[serde(default)]
    pub tool_tags: Vec<String>,
    pub requires_phased: bool,
    pub supports_gl_input: bool,
    pub supports_minimac_m3vcf: bool,
    pub glimpse_reference_format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapCompatibility {
    #[serde(default)]
    pub tool_tags: Vec<String>,
    pub coordinate_system: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PanelLockEntry {
    pub species_id: String,
    pub build_id: String,
    pub panel_id: String,
    pub version: String,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapLockEntry {
    pub species_id: String,
    pub build_id: String,
    pub map_id: String,
    pub version: String,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
}
