//! Owner: bijux-dna-bench
//! Verified publication of immutable governed data catalog registries.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_governed_data_catalog_document, validate_governed_data_catalog_registry,
    GovernedDataCatalogDocument, GovernedDataCatalogRegistry,
};
use serde::Serialize;

const MAX_REGISTRY_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernedDataCatalogRegistryPublicationRequest {
    pub data_root: PathBuf,
    pub source_commit: String,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct GovernedDataCatalogRegistryPublication {
    pub registry_path: PathBuf,
    pub registry_sha256: String,
    pub registry: GovernedDataCatalogRegistry,
}

/// Verify an explicit registry candidate and publish its immutable source-addressed authority.
///
/// # Errors
/// Returns an error for an invalid commit, unsafe path, registry or snapshot drift, or a
/// conflicting existing publication.
pub fn publish_governed_data_catalog_registry(
    request: &GovernedDataCatalogRegistryPublicationRequest,
) -> Result<GovernedDataCatalogRegistryPublication> {
    require_git_commit(&request.source_commit)?;
    let data_root = canonical_directory(&request.data_root, "governed data root")?;
    let candidate_path = data_root
        .join("evidence/catalog-registries")
        .join(&request.source_commit)
        .join("registry.json");
    let registry = load_registry(&candidate_path)?;
    verify_registry_closure(&registry, &data_root)?;

    let registry_path =
        data_root.join("catalog/registries").join(format!("{}.json", request.source_commit));
    match fs::symlink_metadata(&registry_path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("published catalog registry is not a regular non-symlink file");
            }
            let published = load_registry(&registry_path)?;
            verify_registry_closure(&published, &data_root)?;
            if published != registry {
                bail!("published catalog registry conflicts with immutable candidate");
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_json(&registry_path, &registry).with_context(|| {
                format!("publish governed data catalog registry {}", registry_path.display())
            })?;
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect catalog registry {}", registry_path.display()));
        }
    }
    let registry_sha256 =
        bijux_dna_infra::hash_file_sha256(&registry_path).context("hash catalog registry")?;
    Ok(GovernedDataCatalogRegistryPublication { registry_path, registry_sha256, registry })
}

fn load_registry(path: &Path) -> Result<GovernedDataCatalogRegistry> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect catalog registry {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("catalog registry is not a regular non-symlink file: {}", path.display());
    }
    let canonical = path
        .canonicalize()
        .with_context(|| format!("canonicalize catalog registry {}", path.display()))?;
    if canonical != path {
        bail!("catalog registry path is not canonical: {}", path.display());
    }
    let bytes = bijux_dna_infra::read_to_end_bounded(path, MAX_REGISTRY_BYTES)
        .with_context(|| format!("read catalog registry {}", path.display()))?;
    let registry = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse catalog registry {}", path.display()))?;
    validate_governed_data_catalog_registry(&registry)
        .context("validate governed data catalog registry")?;
    Ok(registry)
}

fn verify_registry_closure(registry: &GovernedDataCatalogRegistry, data_root: &Path) -> Result<()> {
    for record in &registry.records {
        for snapshot in &record.snapshots {
            let relative = snapshot
                .catalog_path
                .strip_prefix("study://data/")
                .context("catalog snapshot URI is outside governed data root")?;
            let path = data_root.join(relative);
            let metadata = fs::symlink_metadata(&path)
                .with_context(|| format!("inspect catalog snapshot {}", path.display()))?;
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("catalog snapshot is not a regular non-symlink file: {}", path.display());
            }
            let canonical = path
                .canonicalize()
                .with_context(|| format!("canonicalize catalog snapshot {}", path.display()))?;
            if canonical != path {
                bail!("catalog snapshot path is not canonical: {}", path.display());
            }
            let observed = bijux_dna_infra::hash_file_sha256(&path)
                .with_context(|| format!("hash catalog snapshot {}", path.display()))?;
            if observed != snapshot.catalog_sha256 {
                bail!("catalog snapshot digest drifted: {}", path.display());
            }
            let bytes = bijux_dna_infra::read_to_end_bounded(&path, MAX_REGISTRY_BYTES)?;
            let value: serde_json::Value = serde_json::from_slice(&bytes)
                .with_context(|| format!("parse catalog snapshot {}", path.display()))?;
            if value.get("id").and_then(serde_json::Value::as_str) != Some(record.id.as_str())
                || value.get("schema_version").and_then(serde_json::Value::as_str)
                    != Some(snapshot.catalog_schema_version.as_str())
            {
                bail!("catalog snapshot identity or schema drifted: {}", path.display());
            }
            if snapshot.catalog_schema_version
                == bijux_dna_bench_model::GOVERNED_DATA_CATALOG_SCHEMA_V2
            {
                let document: GovernedDataCatalogDocument = serde_json::from_slice(&bytes)
                    .with_context(|| format!("parse v2 catalog snapshot {}", path.display()))?;
                validate_governed_data_catalog_document(&document)
                    .with_context(|| format!("validate v2 catalog snapshot {}", path.display()))?;
            }
        }
    }
    Ok(())
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not a regular non-symlink directory");
    }
    let canonical = path.canonicalize().with_context(|| format!("canonicalize {label}"))?;
    if canonical != path {
        bail!("{label} path is not canonical");
    }
    Ok(canonical)
}

fn require_git_commit(value: &str) -> Result<()> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("source commit must be a lowercase 40-character Git object id");
    }
    Ok(())
}
