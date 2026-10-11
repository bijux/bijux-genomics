use super::super::{load_container_catalog, ContainerCatalog, CONTAINER_CATALOG_SCHEMA_V1};
use anyhow::{ensure, Result};
use std::collections::BTreeSet;
use std::path::Path;
/// Admitted delivery view of the complete canonical container catalog.
/// The source model remains authoritative; this view does not parse a reduced schema.
pub struct ContainerDeliveryCatalog {
    pub(super) catalog: ContainerCatalog,
}
impl ContainerDeliveryCatalog {
    /// Admit unique, nonempty tool identities from a canonical catalog.
    ///
    /// # Errors
    /// Rejects blank, padded or repeated tool identities.
    pub fn from_catalog(catalog: ContainerCatalog) -> Result<Self> {
        ensure!(
            catalog.schema_version == CONTAINER_CATALOG_SCHEMA_V1,
            "unsupported container catalog schema: {}",
            catalog.schema_version
        );
        let mut ids = BTreeSet::new();
        for tool in &catalog.tools {
            ensure!(
                !tool.id.is_empty() && tool.id.trim() == tool.id,
                "container delivery tool identity must be nonempty and unpadded"
            );
            ensure!(
                ids.insert(tool.id.as_str()),
                "container catalog repeats tool id `{}`",
                tool.id
            );
        }
        Ok(Self { catalog })
    }
    /// Load and admit the complete canonical catalog.
    ///
    /// # Errors
    /// Rejects unreadable or malformed catalogs and invalid tool identities.
    pub fn load(repo_root: &Path) -> Result<Self> {
        Self::from_catalog(load_container_catalog(repo_root)?)
    }
}
