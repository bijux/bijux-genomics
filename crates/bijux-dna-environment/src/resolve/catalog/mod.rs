use std::collections::HashMap;
use std::path::Path;

use super::types::RegistryImagePinFile;
use super::{EnvError, PlatformSpec, ResolvedImage, ToolImageSpec};

/// Resolve an image reference for a tool and platform.
///
/// # Errors
/// Returns an error if the tool name violates image naming rules.
pub(super) fn resolve_image(
    tool: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage, EnvError> {
    if tool.tool.to_lowercase().contains("base") {
        return Err(EnvError::Image(format!(
            "tool image name must not reference base: {}",
            tool.tool
        )));
    }
    let full_name = if let Some(digest) = tool.digest.as_ref() {
        format!("{}/{}@{}", platform.image_prefix, tool.tool, digest)
    } else {
        format!(
            "{}/{}:{}-{}",
            platform.image_prefix, tool.tool, tool.version, platform.arch
        )
    };
    Ok(ResolvedImage {
        full_name,
        arch: platform.arch.clone(),
        runner: platform.runner,
    })
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

pub(super) fn hydrate_catalog_digests_from_registry(
    catalog: &mut HashMap<String, ToolImageSpec>,
    registry_path: &Path,
) -> Result<(), EnvError> {
    if !registry_path.is_file() {
        return Ok(());
    }
    let contents = std::fs::read_to_string(registry_path)?;
    let registry: RegistryImagePinFile = bijux_dna_infra::formats::parse_toml(&contents)
        .map_err(|err| EnvError::Parse(err.message))?;
    for tool in registry.tools {
        if tool.id.trim().is_empty() {
            continue;
        }
        let Some(container_ref) = tool.container_ref.as_deref() else {
            continue;
        };
        let Some((_, digest)) = container_ref.split_once('@') else {
            continue;
        };
        let Some(digest) = digest.strip_prefix("sha256:") else {
            continue;
        };
        let Some(spec) = catalog.get_mut(&tool.id) else {
            continue;
        };
        if spec.digest.is_none() {
            spec.digest = Some(format!("sha256:{digest}"));
        }
    }
    Ok(())
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
