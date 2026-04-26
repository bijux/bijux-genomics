use std::collections::HashMap;
use std::path::Path;

use super::super::types::RegistryImagePinFile;
use super::super::{EnvError, ToolImageSpec};

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
        let digest = digest.trim();
        if digest.is_empty() {
            continue;
        }
        let Some(spec) = catalog.get_mut(&tool.id) else {
            continue;
        };
        if spec.digest.is_none() {
            spec.digest = Some(format!("sha256:{digest}"));
        }
    }
    Ok(())
}
