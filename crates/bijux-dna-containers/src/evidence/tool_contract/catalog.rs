use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

pub const CONTAINER_CATALOG_SCHEMA_V1: &str = "bijux.container.catalog.v1";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerCatalog {
    #[serde(deserialize_with = "deserialize_catalog_schema_version")]
    #[schemars(schema_with = "catalog_schema_version")]
    pub schema_version: String,
    pub owner: String,
    pub contact: String,
    pub repository: String,
    pub apptainer_architecture: String,
    pub docker_architecture: String,
    pub evidence_schema: String,
    pub recipe_license: String,
    pub required_checks: Vec<String>,
    pub evidence_dimensions: Vec<String>,
    pub tools: Vec<ContainerCatalogTool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
// These independent capability and network-policy fields mirror the governed TOML contract.
#[allow(clippy::struct_excessive_bools)]
pub struct ContainerCatalogTool {
    pub id: String,
    pub display_name: String,
    pub purpose: String,
    pub version: String,
    pub support_disposition: String,
    #[serde(default)]
    pub support_reason: String,
    pub upstream: String,
    pub documentation: String,
    pub license: String,
    #[serde(default)]
    pub source_sha256: String,
    #[serde(default)]
    pub pinned_commit: String,
    pub domains: Vec<String>,
    pub stage_ids: Vec<String>,
    pub expected_bin: String,
    pub version_command: String,
    pub help_command: String,
    pub expected_version_regex: String,
    pub apptainer_definition: String,
    pub apptainer_supported: bool,
    pub dockerfile: String,
    pub docker_arm64_supported: bool,
    #[serde(default)]
    pub docker_arm64_reason: String,
    pub build_network: bool,
    pub runtime_network: bool,
    pub real_test_stage: String,
    pub real_test_input_kind: String,
}

pub(super) struct CatalogToolSource {
    pub tool: ContainerCatalogTool,
    pub catalog_bytes: Vec<u8>,
    pub recipe_path: PathBuf,
    pub recipe_bytes: Vec<u8>,
}

/// Load the canonical repository container catalog without projecting away contract fields.
///
/// # Errors
/// Returns an error when the catalog is missing, malformed, or uses an unsupported schema.
pub fn load_container_catalog(repository_root: &Path) -> Result<ContainerCatalog> {
    let catalog_path = repository_root.join("containers/contracts/catalog.toml");
    let catalog_bytes = fs::read(&catalog_path)
        .with_context(|| format!("read container catalog {}", catalog_path.display()))?;
    let catalog: ContainerCatalog = toml::from_slice(&catalog_bytes)
        .with_context(|| format!("parse container catalog {}", catalog_path.display()))?;
    if catalog.schema_version != CONTAINER_CATALOG_SCHEMA_V1 {
        bail!("unsupported container catalog schema: {}", catalog.schema_version);
    }
    Ok(catalog)
}

/// Return every retained Apptainer tool ID in canonical catalog order.
///
/// # Errors
/// Returns an error when the catalog cannot be read, parsed, or matched to the architecture.
pub fn retained_apptainer_tool_ids(
    repository_root: &Path,
    architecture: &str,
) -> Result<Vec<String>> {
    let catalog = load_container_catalog(repository_root)?;
    validate_catalog_header(&catalog, architecture)?;
    let tools = catalog
        .tools
        .into_iter()
        .filter(|tool| tool.support_disposition == "retained" && tool.apptainer_supported)
        .map(|tool| tool.id)
        .collect::<Vec<_>>();
    if tools.is_empty() {
        bail!("container catalog has no retained Apptainer tools");
    }
    let unique = tools.iter().collect::<BTreeSet<_>>();
    if unique.len() != tools.len() {
        bail!("container catalog has duplicate retained Apptainer tool IDs");
    }
    Ok(tools)
}

/// Hash the canonical container catalog exactly as stored at a repository commit.
///
/// # Errors
/// Returns an error when the commit identity is malformed, Git cannot read the catalog blob, or
/// the repository does not contain the canonical catalog at that commit.
pub fn catalog_identity_at_commit(repository_root: &Path, commit: &str) -> Result<String> {
    validate_commit_identity(commit)?;
    let object = format!("{commit}:containers/contracts/catalog.toml");
    let output = Command::new("git")
        .arg("-C")
        .arg(repository_root)
        .args(["show", "--no-ext-diff", &object])
        .output()
        .with_context(|| format!("read container catalog at repository commit {commit}"))?;
    if !output.status.success() {
        bail!(
            "cannot read container catalog at repository commit {commit}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(bijux_dna_infra::hash_bytes_sha256(&output.stdout))
}

pub(super) fn load_retained_tool(
    repository_root: &Path,
    tool_id: &str,
    architecture: &str,
) -> Result<CatalogToolSource> {
    let catalog_path = repository_root.join("containers/contracts/catalog.toml");
    let catalog_bytes = fs::read(&catalog_path)
        .with_context(|| format!("read container catalog {}", catalog_path.display()))?;
    let catalog: ContainerCatalog = toml::from_slice(&catalog_bytes)
        .with_context(|| format!("parse container catalog {}", catalog_path.display()))?;
    validate_catalog_header(&catalog, architecture)?;
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

fn validate_catalog_header(catalog: &ContainerCatalog, architecture: &str) -> Result<()> {
    if catalog.schema_version != CONTAINER_CATALOG_SCHEMA_V1 {
        bail!("unsupported container catalog schema: {}", catalog.schema_version);
    }
    if architecture != catalog.apptainer_architecture {
        bail!(
            "requested architecture {architecture} differs from catalog architecture {}",
            catalog.apptainer_architecture
        );
    }
    Ok(())
}

fn catalog_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": CONTAINER_CATALOG_SCHEMA_V1,
    })
}

fn deserialize_catalog_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == CONTAINER_CATALOG_SCHEMA_V1 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported container catalog schema: {value}")))
    }
}

pub(super) fn validate_commit_identity(commit: &str) -> Result<()> {
    if commit.len() != 40
        || !commit.bytes().all(|byte| byte.is_ascii_hexdigit())
        || commit.bytes().all(|byte| byte == b'0')
    {
        bail!("invalid repository commit identity");
    }
    Ok(())
}
