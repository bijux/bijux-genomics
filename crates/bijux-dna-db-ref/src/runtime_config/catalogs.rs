use std::collections::BTreeMap;

use serde::Deserialize;

use crate::{MapCatalogEntry, MapLockEntry, PanelCatalogEntry, PanelLockEntry};

#[derive(Debug, Deserialize)]
pub(crate) struct PanelsConfig {
    #[serde(default)]
    pub(crate) panel: Vec<PanelCatalogEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MapsConfig {
    #[serde(default)]
    pub(crate) map: Vec<MapCatalogEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PanelLocksConfig {
    #[serde(default)]
    pub(crate) locks: BTreeMap<String, PanelLockEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MapLocksConfig {
    #[serde(default)]
    pub(crate) locks: BTreeMap<String, MapLockEntry>,
}
