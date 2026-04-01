use std::collections::HashMap;
use std::path::Path;

use super::super::{EnvError, ToolImageSpec};
use super::registry_hydration::hydrate_catalog_digests_from_registry;

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

/// Load tool images from a specific TOML file.
///
/// # Errors
/// Returns an error if the file cannot be read, parsed, or contains invalid entries.
pub(super) fn load_image_catalog_from_file(
    path: &Path,
) -> Result<HashMap<String, ToolImageSpec>, EnvError> {
    let contents = std::fs::read_to_string(path)?;
    let raw: HashMap<String, ToolImageSpec> = bijux_dna_infra::formats::parse_toml(&contents)
        .map_err(|err| EnvError::Parse(err.message))?;
    let mut catalog = HashMap::new();
    for (key, mut spec) in raw {
        if key.trim().is_empty() {
            return Err(EnvError::Image(
                "empty tool name in images.toml".to_string(),
            ));
        }
        if spec.version.trim().is_empty() {
            return Err(EnvError::Image(format!("empty version for tool {key}")));
        }
        if spec.tool.trim().is_empty() {
            spec.tool.clone_from(&key);
        }
        if catalog.insert(key.clone(), spec).is_some() {
            return Err(EnvError::Image(format!("duplicate tool {key}")));
        }
    }
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
