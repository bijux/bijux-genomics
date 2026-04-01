use std::collections::HashMap;
use std::path::Path;

use super::super::{EnvError, ToolImageSpec};
use super::catalog_loader::load_image_catalog_from_file;
use super::registry_hydration::hydrate_catalog_digests_from_registry;

mod catalog_loader;
mod image_resolution;
mod registry_hydration;

pub(super) use image_resolution::resolve_image;

/// Load tool images from configs/ci/tools/images.toml.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or contains invalid entries.
pub(super) fn load_image_catalog() -> Result<HashMap<String, ToolImageSpec>, EnvError> {
    let path = bijux_dna_infra::configs_file(Path::new("."), "ci/tools/images.toml");
    let mut catalog = load_image_catalog_from_file(&path)?;
    let registry_path =
        bijux_dna_infra::configs_file(Path::new("."), "ci/registry/tool_registry.toml");
    hydrate_catalog_digests_from_registry(&mut catalog, &registry_path)?;
    Ok(catalog)
}

/// Validate that tools have image entries.
///
/// # Errors
/// Returns an error if any tool is missing from the catalog.
#[allow(clippy::implicit_hasher)]
pub(super) fn validate_images_for_stage(
    catalog: &HashMap<String, ToolImageSpec>,
    tools: &[&str],
) -> Result<(), EnvError> {
    for tool in tools {
        if !catalog.contains_key(*tool) {
            return Err(EnvError::Image(format!(
                "missing image entry for tool {tool}"
            )));
        }
    }
    Ok(())
}
