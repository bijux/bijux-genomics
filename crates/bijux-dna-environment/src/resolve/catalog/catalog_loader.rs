use std::collections::HashMap;
use std::path::Path;

use super::super::{EnvError, ToolImageSpec};

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
