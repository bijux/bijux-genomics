use std::collections::HashMap;

use super::{EnvError, PlatformSpec, ResolvedImage, ToolImageSpec};

/// Resolver entrypoint for environment specs and image catalog.
#[derive(Debug, Default, Clone, Copy)]
pub struct EnvironmentResolver;

impl EnvironmentResolver {
    /// # Errors
    /// Returns an error if the platform cannot be loaded.
    pub fn load_platform(name: Option<&str>) -> Result<PlatformSpec, EnvError> {
        super::entrypoints::load_platform(name)
    }

    /// # Errors
    /// Returns an error if the image catalog cannot be loaded.
    pub fn load_image_catalog() -> Result<HashMap<String, ToolImageSpec>, EnvError> {
        super::entrypoints::load_image_catalog()
    }

    /// # Errors
    /// Returns an error if the image cannot be resolved.
    pub fn resolve_image(
        tool: &ToolImageSpec,
        platform: &PlatformSpec,
    ) -> Result<ResolvedImage, EnvError> {
        super::entrypoints::resolve_image(tool, platform)
    }

    /// # Errors
    /// Returns an error if validation fails.
    pub fn validate_images_for_stage(
        catalog: &HashMap<String, ToolImageSpec>,
        tools: &[&str],
    ) -> Result<(), EnvError> {
        super::entrypoints::validate_images_for_stage(catalog, tools)
    }
}
