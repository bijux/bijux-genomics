//! Delivery selection and external runtime admission over the canonical catalog schema.
mod admission;
mod model;
mod projection;
pub use admission::ContainerDeliveryCatalog;
use anyhow::Result;
pub use model::ExternalArtifactToolContract;
use std::{collections::BTreeSet, path::Path};
/// Load all supported tools without excluding operator-provided external artifacts.
///
/// # Errors
/// Rejects malformed catalogs and absent supported tools.
pub fn load_benchmark_integrated_container_tool_ids(repo_root: &Path) -> Result<BTreeSet<String>> {
    ContainerDeliveryCatalog::load(repo_root)?.supported_tool_ids()
}
/// Load admitted external runtime verification contracts.
///
/// # Errors
/// Rejects malformed catalogs, invalid delivery policy or incomplete runtime metadata.
pub fn load_external_artifact_tool_contracts(
    repo_root: &Path,
) -> Result<Vec<ExternalArtifactToolContract>> {
    ContainerDeliveryCatalog::load(repo_root)?.external_artifact_contracts()
}
/// Load exact external stage/tool binding identities.
///
/// # Errors
/// Returns catalog or external delivery admission failures.
pub fn load_external_artifact_binding_keys(repo_root: &Path) -> Result<BTreeSet<(String, String)>> {
    ContainerDeliveryCatalog::load(repo_root)?.external_artifact_binding_keys()
}
