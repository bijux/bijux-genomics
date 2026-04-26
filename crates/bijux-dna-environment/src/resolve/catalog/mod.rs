use std::collections::HashMap;
use std::path::Path;

use self::catalog_loader::load_image_catalog_from_file;
use self::registry_hydration::hydrate_catalog_digests_from_registry;
use super::{EnvError, PlatformSpec, ResolvedImage, ToolImageSpec};

mod catalog_loader;
mod image_resolution;
mod registry_hydration;

/// Resolve an image reference for a tool and platform.
///
/// # Errors
/// Returns an error if the tool name violates image naming rules.
pub(super) fn resolve_image(
    tool: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage, EnvError> {
    image_resolution::resolve_image(tool, platform)
}

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
        let Some(spec) = catalog.get(*tool) else {
            return Err(EnvError::Image(format!("missing image entry for tool {tool}")));
        };
        if spec.enabled == Some(false) {
            return Err(EnvError::Image(format!("image entry for tool {tool} is disabled")));
        }
    }
    Ok(())
}
