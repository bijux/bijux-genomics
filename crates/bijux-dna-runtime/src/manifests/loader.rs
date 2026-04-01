use std::path::Path;

use anyhow::{anyhow, Result};

use bijux_dna_core::contract::ToolRegistry;

use super::domain_registry::read_domain_registry;
use super::generated_registry::read_generated_registry;
use super::source::find_domain_dir;

/// # Errors
/// Returns an error if registry config cannot be read or parsed.
pub fn load_manifests(source_path: &Path) -> Result<ToolRegistry> {
    if let Some(domain_dir) = find_domain_dir(source_path) {
        return read_domain_registry(&domain_dir);
    }

    let registry_path = if source_path.is_dir() {
        bijux_dna_infra::configs_file(source_path, "ci/registry/tool_registry.toml")
    } else {
        source_path.to_path_buf()
    };
    if !registry_path.exists() {
        return Err(anyhow!(
            "registry file {} does not exist",
            registry_path.display()
        ));
    }
    read_generated_registry(&registry_path)
}
