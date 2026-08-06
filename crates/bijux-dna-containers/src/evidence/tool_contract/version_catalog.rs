//! Typed source identities and portable map derivation for container tool versions.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use anyhow::{ensure, Context, Result};
use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};

use super::load_container_catalog;

pub const CONTAINER_VERSION_MAP_SCHEMA_V2: &str = "bijux.container.version_map.v2";
pub const CONTAINER_VERSION_CATALOG_PATH: &str = "containers/versions/versions.toml";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerVersionMap {
    #[serde(deserialize_with = "deserialize_version_map_schema_version")]
    #[schemars(schema_with = "version_map_schema_version")]
    pub schema_version: String,
    pub source: String,
    pub items: Vec<ContainerVersionMapItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContainerVersionMapItem {
    pub tool: String,
    pub version: String,
    pub source: String,
    pub source_sha256: String,
    pub pinned_commit: String,
    pub date_pinned: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
struct ContainerVersionSource {
    version: String,
    source: String,
    date_pinned: String,
    #[serde(default)]
    source_sha256: String,
    #[serde(default)]
    pinned_commit: String,
    #[serde(default)]
    upstream_license: String,
    #[serde(default)]
    linux_amd64_package_sha256: String,
    #[serde(default)]
    linux_arm64_package_sha256: String,
    #[serde(default)]
    linux_amd64_pcre_sha256: String,
    #[serde(default)]
    linux_arm64_pcre_sha256: String,
}

/// Derive the portable version map from source identities reconciled with the canonical catalog.
///
/// # Errors
/// Returns an error for unknown fields, roster drift, version drift, shared commit drift, or
/// malformed source identities.
pub fn derive_container_version_map(repository_root: &Path) -> Result<ContainerVersionMap> {
    let source_path = repository_root.join(CONTAINER_VERSION_CATALOG_PATH);
    let source_bytes = fs::read(&source_path)
        .with_context(|| format!("read container version catalog {}", source_path.display()))?;
    let versions: BTreeMap<String, ContainerVersionSource> = toml::from_slice(&source_bytes)
        .with_context(|| format!("parse container version catalog {}", source_path.display()))?;
    let catalog = load_container_catalog(repository_root)?;
    let catalog_ids = catalog.tools.iter().map(|tool| tool.id.as_str()).collect::<BTreeSet<_>>();
    let version_ids = versions.keys().map(String::as_str).collect::<BTreeSet<_>>();
    ensure!(
        version_ids == catalog_ids,
        "container version catalog roster differs from canonical container catalog"
    );

    let mut items = Vec::with_capacity(versions.len());
    for tool in &catalog.tools {
        let version = versions
            .get(&tool.id)
            .with_context(|| format!("container version catalog omits {}", tool.id))?;
        validate_source(tool.id.as_str(), version)?;
        ensure!(
            version.version == tool.version,
            "{} version differs between version and container catalogs",
            tool.id
        );
        ensure!(
            version.pinned_commit.is_empty()
                || tool.pinned_commit.is_empty()
                || version.pinned_commit == tool.pinned_commit,
            "{} pinned commit differs between version and container catalogs",
            tool.id
        );
        items.push(ContainerVersionMapItem {
            tool: tool.id.clone(),
            version: version.version.clone(),
            source: version.source.clone(),
            source_sha256: version.source_sha256.clone(),
            pinned_commit: version.pinned_commit.clone(),
            date_pinned: version.date_pinned.clone(),
        });
    }

    Ok(ContainerVersionMap {
        schema_version: CONTAINER_VERSION_MAP_SCHEMA_V2.to_string(),
        source: CONTAINER_VERSION_CATALOG_PATH.to_string(),
        items,
    })
}

/// Render the canonical portable version map with stable object-key order and a trailing newline.
///
/// # Errors
/// Returns an error when source admission or JSON serialization fails.
pub fn render_container_version_map(repository_root: &Path) -> Result<String> {
    let value = serde_json::to_value(derive_container_version_map(repository_root)?)?;
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}

fn validate_source(tool_id: &str, source: &ContainerVersionSource) -> Result<()> {
    ensure!(!source.version.trim().is_empty(), "{tool_id}: version is empty");
    ensure!(
        source.source.starts_with("https://") || source.source.starts_with("http://"),
        "{tool_id}: source is not an HTTP(S) URL"
    );
    ensure!(!source.date_pinned.trim().is_empty(), "{tool_id}: date_pinned is empty");
    for (name, digest) in [
        ("source_sha256", source.source_sha256.as_str()),
        ("linux_amd64_package_sha256", source.linux_amd64_package_sha256.as_str()),
        ("linux_arm64_package_sha256", source.linux_arm64_package_sha256.as_str()),
        ("linux_amd64_pcre_sha256", source.linux_amd64_pcre_sha256.as_str()),
        ("linux_arm64_pcre_sha256", source.linux_arm64_pcre_sha256.as_str()),
    ] {
        ensure!(
            digest.is_empty()
                || (digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())),
            "{tool_id}: {name} is not a SHA-256 identity"
        );
    }
    ensure!(
        source.pinned_commit.is_empty()
            || (matches!(source.pinned_commit.len(), 7 | 40)
                && source.pinned_commit.bytes().all(|byte| byte.is_ascii_hexdigit())),
        "{tool_id}: pinned_commit is not a Git identity"
    );
    ensure!(
        source.upstream_license == source.upstream_license.trim(),
        "{tool_id}: upstream_license has surrounding whitespace"
    );
    Ok(())
}

fn version_map_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({
        "type": "string",
        "const": CONTAINER_VERSION_MAP_SCHEMA_V2,
    })
}

fn deserialize_version_map_schema_version<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    if value == CONTAINER_VERSION_MAP_SCHEMA_V2 {
        Ok(value)
    } else {
        Err(D::Error::custom(format!("unsupported container version map schema: {value}")))
    }
}
