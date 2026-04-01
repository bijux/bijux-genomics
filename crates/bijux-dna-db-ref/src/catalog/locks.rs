use serde::{Deserialize, Serialize};

use super::CatalogFileEntry;

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
