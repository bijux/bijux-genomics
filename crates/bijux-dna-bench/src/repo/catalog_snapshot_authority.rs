//! Owner: bijux-dna-bench
//! Immutable preservation and registry composition for heterogeneous governed data catalogs.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_governed_data_catalog_registry, validate_governed_data_catalog_snapshot_publication,
    validate_governed_preserved_data_catalog, validate_utc_timestamp, GovernedDataCatalogRegistry,
    GovernedDataCatalogRegistryRecord, GovernedDataCatalogSnapshot, GovernedPreservedDataCatalog,
    GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SCHEMA_V2, GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1,
};

pub use bijux_dna_bench_model::GovernedDataCatalogSnapshotPublication;
use serde::Serialize;

const MAX_CATALOG_BYTES: usize = 16 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GovernedDataCatalogSnapshotRequest {
    pub data_root: PathBuf,
    pub source_commit: String,
    pub preserved_at_utc: String,
    pub previous_registry_path: Option<PathBuf>,
}

/// Preserve every flat v1 authority under a source-addressed v2 envelope and compose a registry.
///
/// Existing current v2 records are retained from the explicit prior registry. Publications are
/// immutable and replay-safe: matching bytes are accepted, while conflicts are rejected.
///
/// # Errors
/// Returns an error for invalid identities, unsafe paths, malformed catalogs, drifted prior
/// snapshots, or conflicting immutable publications.
pub fn publish_governed_data_catalog_snapshots(
    request: &GovernedDataCatalogSnapshotRequest,
) -> Result<GovernedDataCatalogSnapshotPublication> {
    require_commit(&request.source_commit)?;
    validate_utc_timestamp(&request.preserved_at_utc, "catalog preservation")?;
    let data_root = canonical_directory(&request.data_root, "governed data root")?;
    let prior = request
        .previous_registry_path
        .as_ref()
        .map(|path| load_prior_registry(path, &data_root))
        .transpose()?;
    let prior_records = prior
        .as_ref()
        .map(|registry| {
            registry
                .records
                .iter()
                .map(|record| (record.id.as_str(), record))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let flat_catalogs = load_flat_catalogs(&data_root)?;
    let flat_ids = flat_catalogs.iter().map(|catalog| catalog.id.as_str()).collect::<BTreeSet<_>>();
    if prior_records.keys().any(|id| !flat_ids.contains(*id)) {
        bail!("prior catalog registry contains an identity absent from flat authorities");
    }

    let mut records = Vec::with_capacity(flat_catalogs.len());
    let mut preserved_snapshot_count = 0_u64;
    let mut retained_snapshot_count = 0_u64;
    for catalog in flat_catalogs {
        if let Some(record) = prior_records.get(catalog.id.as_str()) {
            records.push((*record).clone());
            retained_snapshot_count = retained_snapshot_count
                .checked_add(1)
                .context("retained catalog count overflow")?;
            continue;
        }

        let preserved = GovernedPreservedDataCatalog {
            schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V2.to_string(),
            id: catalog.id.clone(),
            source_catalog_path: format!("study://data/catalog/{}.json", catalog.id),
            source_catalog_sha256: catalog.sha256,
            source_catalog_size_bytes: catalog.size_bytes,
            source_catalog_schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V1.to_string(),
            preserved_at_utc: request.preserved_at_utc.clone(),
            preserved_catalog: catalog.value,
        };
        validate_governed_preserved_data_catalog(&preserved)
            .with_context(|| format!("validate preserved catalog {}", preserved.id))?;
        let snapshot_path = data_root
            .join("catalog/snapshots")
            .join(&request.source_commit)
            .join(format!("{}.json", preserved.id));
        write_immutable_json(&snapshot_path, &preserved, "catalog snapshot")?;
        let snapshot_sha256 = bijux_dna_infra::hash_file_sha256(&snapshot_path)?;
        records.push(GovernedDataCatalogRegistryRecord {
            id: preserved.id.clone(),
            current_catalog_sha256: snapshot_sha256.clone(),
            snapshots: vec![GovernedDataCatalogSnapshot {
                catalog_path: format!(
                    "study://data/catalog/snapshots/{}/{}.json",
                    request.source_commit, preserved.id
                ),
                catalog_sha256: snapshot_sha256,
                catalog_schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V2.to_string(),
                source_commit: request.source_commit.clone(),
                created_at_utc: request.preserved_at_utc.clone(),
            }],
        });
        preserved_snapshot_count =
            preserved_snapshot_count.checked_add(1).context("preserved catalog count overflow")?;
    }
    records.sort_by(|left, right| left.id.cmp(&right.id));
    let registry = GovernedDataCatalogRegistry {
        schema_version: GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1.to_string(),
        created_at_utc: request.preserved_at_utc.clone(),
        data_root: "study://data".to_string(),
        records,
    };
    validate_governed_data_catalog_registry(&registry)
        .context("validate composed governed data catalog registry")?;
    let candidate_registry_path = data_root
        .join("evidence/catalog-registries")
        .join(&request.source_commit)
        .join("registry.json");
    write_immutable_json(&candidate_registry_path, &registry, "catalog registry candidate")?;
    let publication = GovernedDataCatalogSnapshotPublication {
        schema_version: GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1.to_string(),
        candidate_registry_path,
        preserved_snapshot_count,
        retained_snapshot_count,
        registry,
    };
    validate_governed_data_catalog_snapshot_publication(&publication)
        .context("validate governed data catalog snapshot publication")?;
    Ok(publication)
}

struct FlatCatalog {
    id: String,
    sha256: String,
    size_bytes: u64,
    value: serde_json::Value,
}

fn load_flat_catalogs(data_root: &Path) -> Result<Vec<FlatCatalog>> {
    let catalog_root = data_root.join("catalog");
    let mut paths = fs::read_dir(&catalog_root)
        .with_context(|| format!("read catalog root {}", catalog_root.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<Vec<_>>>()?;
    paths.sort();
    let mut catalogs = Vec::new();
    let mut ids = BTreeSet::new();
    for path in paths {
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            bail!("flat catalog is not a regular non-symlink file: {}", path.display());
        }
        let bytes = bijux_dna_infra::read_to_end_bounded(&path, MAX_CATALOG_BYTES)?;
        let value: serde_json::Value = serde_json::from_slice(&bytes)
            .with_context(|| format!("parse flat catalog {}", path.display()))?;
        let object = value.as_object().context("flat catalog root is not an object")?;
        if object.get("schema_version").and_then(serde_json::Value::as_str)
            == Some(GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1)
        {
            continue;
        }
        let id = object
            .get("id")
            .and_then(serde_json::Value::as_str)
            .context("flat catalog lacks an ID")?;
        if object.get("schema_version").and_then(serde_json::Value::as_str)
            != Some(GOVERNED_DATA_CATALOG_SCHEMA_V1)
            || path.file_stem().and_then(|value| value.to_str()) != Some(id)
            || !ids.insert(id.to_string())
        {
            bail!("flat catalog identity or schema is invalid: {}", path.display());
        }
        catalogs.push(FlatCatalog {
            id: id.to_string(),
            sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
            size_bytes: metadata.len(),
            value,
        });
    }
    if catalogs.is_empty() {
        bail!("governed data root contains no flat catalog authorities");
    }
    Ok(catalogs)
}

fn load_prior_registry(path: &Path, data_root: &Path) -> Result<GovernedDataCatalogRegistry> {
    let path = canonical_file(path, data_root, "prior catalog registry")?;
    let bytes = bijux_dna_infra::read_to_end_bounded(&path, MAX_CATALOG_BYTES)?;
    let registry: GovernedDataCatalogRegistry = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse prior catalog registry {}", path.display()))?;
    validate_governed_data_catalog_registry(&registry)
        .context("validate prior catalog registry")?;
    for record in &registry.records {
        for snapshot in &record.snapshots {
            let relative = snapshot
                .catalog_path
                .strip_prefix("study://data/")
                .context("prior snapshot URI escapes governed data root")?;
            let snapshot_path =
                canonical_file(&data_root.join(relative), data_root, "prior snapshot")?;
            if bijux_dna_infra::hash_file_sha256(&snapshot_path)? != snapshot.catalog_sha256 {
                bail!("prior catalog snapshot digest drifted: {}", snapshot_path.display());
            }
        }
    }
    Ok(registry)
}

fn write_immutable_json(path: &Path, value: &impl Serialize, label: &str) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("existing {label} is not a regular non-symlink file: {}", path.display());
            }
            if fs::read(path)? != bytes {
                bail!("existing {label} conflicts with immutable publication: {}", path.display());
            }
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, &bytes)
                .with_context(|| format!("publish {label} {}", path.display()))?;
        }
        Err(error) => {
            return Err(error).with_context(|| format!("inspect {label} {}", path.display()))
        }
    }
    Ok(())
}

fn canonical_directory(path: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path).with_context(|| format!("inspect {label}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("{label} is not a regular non-symlink directory");
    }
    path.canonicalize().with_context(|| format!("canonicalize {label}"))
}

fn canonical_file(path: &Path, root: &Path, label: &str) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("inspect {label} {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("{label} is not a regular non-symlink file: {}", path.display());
    }
    let canonical = path.canonicalize()?;
    if !canonical.starts_with(root) {
        bail!("{label} escapes governed data root: {}", path.display());
    }
    Ok(canonical)
}

fn require_commit(value: &str) -> Result<()> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("source commit must be a lowercase 40-character Git commit");
    }
    Ok(())
}
