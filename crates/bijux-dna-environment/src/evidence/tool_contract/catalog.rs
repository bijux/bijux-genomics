use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Catalog {
    schema_version: String,
    apptainer_architecture: String,
    tools: Vec<CatalogTool>,
}

#[derive(Debug, Clone, Deserialize)]
pub(super) struct CatalogTool {
    pub id: String,
    pub version: String,
    pub support_disposition: String,
    #[serde(default)]
    pub source_sha256: String,
    #[serde(default)]
    pub pinned_commit: String,
    pub expected_bin: String,
    pub version_command: String,
    pub expected_version_regex: String,
    pub apptainer_definition: String,
    pub apptainer_supported: bool,
}

pub(super) struct CatalogToolSource {
    pub tool: CatalogTool,
    pub catalog_bytes: Vec<u8>,
    pub recipe_path: PathBuf,
    pub recipe_bytes: Vec<u8>,
}

pub(super) fn load_retained_tool(
    repository_root: &Path,
    tool_id: &str,
    architecture: &str,
) -> Result<CatalogToolSource> {
    let catalog_path = repository_root.join("containers/contracts/catalog.toml");
    let catalog_bytes = fs::read(&catalog_path)
        .with_context(|| format!("read container catalog {}", catalog_path.display()))?;
    let catalog: Catalog = toml::from_slice(&catalog_bytes)
        .with_context(|| format!("parse container catalog {}", catalog_path.display()))?;
    if catalog.schema_version != "bijux.container.catalog.v1" {
        bail!("unsupported container catalog schema: {}", catalog.schema_version);
    }
    if architecture != catalog.apptainer_architecture {
        bail!(
            "requested architecture {architecture} differs from catalog architecture {}",
            catalog.apptainer_architecture
        );
    }
    let tool = catalog
        .tools
        .into_iter()
        .find(|tool| tool.id == tool_id)
        .ok_or_else(|| anyhow::anyhow!("container catalog omits tool {tool_id}"))?;
    if tool.support_disposition != "retained" || !tool.apptainer_supported {
        bail!("tool {tool_id} is not retained for Apptainer packaging");
    }
    if tool.apptainer_definition.trim().is_empty() {
        bail!("retained tool {tool_id} has no Apptainer definition");
    }
    let recipe_path = repository_root.join(&tool.apptainer_definition);
    let recipe_bytes = fs::read(&recipe_path)
        .with_context(|| format!("read Apptainer definition {}", recipe_path.display()))?;
    Ok(CatalogToolSource { tool, catalog_bytes, recipe_path, recipe_bytes })
}
