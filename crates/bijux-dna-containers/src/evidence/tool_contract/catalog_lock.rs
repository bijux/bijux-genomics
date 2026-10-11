//! Immutable catalog identity derived from the complete declarative tool catalog.

use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use super::{load_container_catalog, validate_container_catalog, ContainerCatalogTool};

pub const CONTAINER_CATALOG_LOCK_SCHEMA_V1: &str = "bijux.container.catalog_lock.v1";
pub const CONTAINER_CATALOG_PATH: &str = "containers/contracts/catalog.toml";
pub const CONTAINER_CATALOG_LOCK_PATH: &str = "containers/contracts/catalog.lock.json";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerCatalogLock {
    #[serde(deserialize_with = "deserialize_catalog_lock_schema_version")]
    #[schemars(schema_with = "catalog_lock_schema_version")]
    pub schema_version: String,
    pub source: String,
    pub source_sha256: String,
    pub owner: String,
    pub contact: String,
    pub repository: String,
    pub apptainer_architecture: String,
    pub docker_architecture: String,
    pub evidence_schema: String,
    pub recipe_license: String,
    pub required_checks: Vec<String>,
    pub evidence_dimensions: Vec<String>,
    pub tools: Vec<ContainerCatalogLockTool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerCatalogLockTool {
    pub id: String,
    pub display_name: String,
    pub version: String,
    pub support_disposition: String,
    pub support_reason: String,
    pub upstream: String,
    pub license: String,
    pub source_sha256: String,
    pub pinned_commit: String,
    pub apptainer_definition: String,
    pub dockerfile: String,
    pub contract_sha256: String,
}

/// Derive the immutable catalog lock from the validated repository catalog.
///
/// # Errors
/// Returns an error when the catalog closure is invalid or a retained tool lacks a concrete
/// source identity.
pub fn derive_container_catalog_lock(repository_root: &Path) -> Result<ContainerCatalogLock> {
    let validation = validate_container_catalog(repository_root)?;
    if !validation.is_valid() {
        bail!("cannot lock invalid container catalog: {}", validation.errors.join("; "));
    }
    if validation.unresolved_source_identities != 0 {
        let catalog = load_container_catalog(repository_root)?;
        let unresolved = catalog
            .tools
            .iter()
            .filter(|tool| tool.support_disposition == "retained")
            .filter(|tool| !has_concrete_source_identity(tool))
            .map(|tool| tool.id.as_str())
            .collect::<Vec<_>>();
        bail!("refusing catalog lock with unresolved source identities: {}", unresolved.join(", "));
    }

    let source_path = repository_root.join(CONTAINER_CATALOG_PATH);
    let source_bytes = fs::read(&source_path)
        .with_context(|| format!("read container catalog {}", source_path.display()))?;
    let catalog = load_container_catalog(repository_root)?;
    let mut tools = catalog.tools.clone();
    tools.sort_by(|left, right| left.id.cmp(&right.id));
    let locked_tools = tools
        .into_iter()
        .map(|tool| {
            let contract_sha256 = bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&tool)?);
            Ok(ContainerCatalogLockTool {
                id: tool.id,
                display_name: tool.display_name,
                version: tool.version,
                support_disposition: tool.support_disposition,
                support_reason: tool.support_reason,
                upstream: tool.upstream,
                license: tool.license,
                source_sha256: tool.source_sha256,
                pinned_commit: tool.pinned_commit,
                apptainer_definition: tool.apptainer_definition,
                dockerfile: tool.dockerfile,
                contract_sha256,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(ContainerCatalogLock {
        schema_version: CONTAINER_CATALOG_LOCK_SCHEMA_V1.to_string(),
        source: CONTAINER_CATALOG_PATH.to_string(),
        source_sha256: bijux_dna_infra::hash_bytes_sha256(&source_bytes),
        owner: catalog.owner,
        contact: catalog.contact,
        repository: catalog.repository,
        apptainer_architecture: catalog.apptainer_architecture,
        docker_architecture: catalog.docker_architecture,
        evidence_schema: catalog.evidence_schema,
        recipe_license: catalog.recipe_license,
        required_checks: catalog.required_checks,
        evidence_dimensions: catalog.evidence_dimensions,
        tools: locked_tools,
    })
}

/// Render the canonical catalog lock with a terminating newline.
///
/// # Errors
/// Returns an error when catalog derivation or serialization fails.
pub fn render_container_catalog_lock(repository_root: &Path) -> Result<String> {
    Ok(format!(
        "{}\n",
        serde_json::to_string_pretty(&derive_container_catalog_lock(repository_root)?)?
    ))
}

/// Load and verify the checked-in catalog lock against the current declarative catalog.
///
/// # Errors
/// Returns an error for malformed, stale, unknown-field, or identity-ambiguous lock content.
pub fn load_container_catalog_lock(repository_root: &Path) -> Result<ContainerCatalogLock> {
    let lock_path = repository_root.join(CONTAINER_CATALOG_LOCK_PATH);
    let lock: ContainerCatalogLock = serde_json::from_slice(
        &fs::read(&lock_path)
            .with_context(|| format!("read catalog lock {}", lock_path.display()))?,
    )
    .with_context(|| format!("parse catalog lock {}", lock_path.display()))?;
    validate_container_catalog_lock(repository_root, &lock)?;
    Ok(lock)
}

/// Verify that a catalog lock is the unique current derivation of the repository catalog.
///
/// # Errors
/// Returns an error when any lock identity or tool contract differs from current authority.
pub fn validate_container_catalog_lock(
    repository_root: &Path,
    lock: &ContainerCatalogLock,
) -> Result<()> {
    let expected = derive_container_catalog_lock(repository_root)?;
    if lock != &expected {
        bail!("container catalog lock differs from the current catalog authority");
    }
    Ok(())
}

fn has_concrete_source_identity(tool: &ContainerCatalogTool) -> bool {
    (!tool.source_sha256.is_empty() && !placeholder_identity(&tool.source_sha256))
        || (!tool.pinned_commit.is_empty() && !placeholder_identity(&tool.pinned_commit))
}

fn placeholder_identity(value: &str) -> bool {
    value.bytes().next().is_some_and(|first| value.bytes().all(|byte| byte == first))
}

fn catalog_lock_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": CONTAINER_CATALOG_LOCK_SCHEMA_V1,
    })
}

fn deserialize_catalog_lock_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == CONTAINER_CATALOG_LOCK_SCHEMA_V1 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported container catalog lock schema: {value}")))
    }
}
