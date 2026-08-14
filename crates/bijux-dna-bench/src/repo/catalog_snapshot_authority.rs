//! Owner: bijux-dna-bench
//! Immutable preservation and registry composition for heterogeneous governed data catalogs.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    validate_governed_data_catalog_registry, validate_governed_data_catalog_snapshot_publication,
    validate_governed_derived_data_catalog, validate_governed_preserved_data_catalog,
    validate_utc_timestamp, GovernedDataCatalogRegistry, GovernedDataCatalogRegistryRecord,
    GovernedDataCatalogSnapshot, GovernedDerivedDataCatalog, GovernedPreservedDataCatalog,
    GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1, GOVERNED_DATA_CATALOG_SCHEMA_V1,
    GOVERNED_DATA_CATALOG_SCHEMA_V2, GOVERNED_DATA_CATALOG_SNAPSHOT_PUBLICATION_SCHEMA_V1,
    GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1,
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

/// Publish every flat authority under a source-addressed snapshot and compose a registry.
///
/// Flat v1 authorities receive a preservation envelope; native derived catalogs retain their
/// schema after their source and file closure is reverified. Existing native records are retained
/// from the explicit prior registry. Publications are immutable and replay-safe: matching bytes
/// are accepted, while conflicts are rejected.
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
        let prepared = prepare_snapshot(catalog, &request.preserved_at_utc)?;
        let prior_record = prior_records.get(prepared.id.as_str()).copied();
        if prior_record
            .map(|record| current_snapshot_matches(record, &prepared, &data_root))
            .transpose()?
            == Some(true)
        {
            let record = prior_record.context("matching prior catalog record disappeared")?;
            records.push((*record).clone());
            retained_snapshot_count = retained_snapshot_count
                .checked_add(1)
                .context("retained catalog count overflow")?;
            continue;
        }

        let snapshot_path = data_root
            .join("catalog/snapshots")
            .join(&request.source_commit)
            .join(format!("{}.json", prepared.id));
        write_immutable_json(&snapshot_path, &prepared.value, "catalog snapshot")?;
        let snapshot_sha256 = bijux_dna_infra::hash_file_sha256(&snapshot_path)?;
        let mut snapshots = prior_record.map(|record| record.snapshots.clone()).unwrap_or_default();
        snapshots.push(GovernedDataCatalogSnapshot {
            catalog_path: format!(
                "study://data/catalog/snapshots/{}/{}.json",
                request.source_commit, prepared.id
            ),
            catalog_sha256: snapshot_sha256.clone(),
            catalog_schema_version: prepared.snapshot_schema_version,
            source_commit: request.source_commit.clone(),
            created_at_utc: request.preserved_at_utc.clone(),
        });
        snapshots.sort_by(|left, right| left.catalog_path.cmp(&right.catalog_path));
        records.push(GovernedDataCatalogRegistryRecord {
            id: prepared.id,
            current_catalog_sha256: snapshot_sha256.clone(),
            snapshots,
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
    schema_version: String,
    sha256: String,
    size_bytes: u64,
    value: serde_json::Value,
}

struct PreparedCatalogSnapshot {
    id: String,
    flat_schema_version: String,
    flat_sha256: String,
    snapshot_schema_version: String,
    value: serde_json::Value,
}

fn prepare_snapshot(
    catalog: FlatCatalog,
    preserved_at_utc: &str,
) -> Result<PreparedCatalogSnapshot> {
    let flat_schema_version = catalog.schema_version.clone();
    let flat_sha256 = catalog.sha256.clone();
    let (snapshot_schema_version, value) = match catalog.schema_version.as_str() {
        GOVERNED_DATA_CATALOG_SCHEMA_V1 => {
            let preserved = GovernedPreservedDataCatalog {
                schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V2.to_string(),
                id: catalog.id.clone(),
                source_catalog_path: format!("study://data/catalog/{}.json", catalog.id),
                source_catalog_sha256: catalog.sha256,
                source_catalog_size_bytes: catalog.size_bytes,
                source_catalog_schema_version: GOVERNED_DATA_CATALOG_SCHEMA_V1.to_string(),
                preserved_at_utc: preserved_at_utc.to_string(),
                preserved_catalog: catalog.value,
            };
            validate_governed_preserved_data_catalog(&preserved)
                .with_context(|| format!("validate preserved catalog {}", preserved.id))?;
            (GOVERNED_DATA_CATALOG_SCHEMA_V2.to_string(), serde_json::to_value(preserved)?)
        }
        GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1 => {
            (GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1.to_string(), catalog.value)
        }
        _ => bail!("flat catalog schema is unsupported: {}", catalog.id),
    };
    Ok(PreparedCatalogSnapshot {
        id: catalog.id,
        flat_schema_version,
        flat_sha256,
        snapshot_schema_version,
        value,
    })
}

fn current_snapshot_matches(
    record: &GovernedDataCatalogRegistryRecord,
    prepared: &PreparedCatalogSnapshot,
    data_root: &Path,
) -> Result<bool> {
    let current = record
        .snapshots
        .iter()
        .find(|snapshot| snapshot.catalog_sha256 == record.current_catalog_sha256)
        .context("prior catalog record lacks its current snapshot")?;
    let relative = current
        .catalog_path
        .strip_prefix("study://data/")
        .context("prior current catalog URI escapes governed data root")?;
    let path = canonical_file(&data_root.join(relative), data_root, "prior current catalog")?;
    let bytes = bijux_dna_infra::read_to_end_bounded(&path, MAX_CATALOG_BYTES)?;
    let value: serde_json::Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("parse prior current catalog {}", path.display()))?;
    match prepared.flat_schema_version.as_str() {
        GOVERNED_DATA_CATALOG_SCHEMA_V1 => Ok(current.catalog_schema_version
            == GOVERNED_DATA_CATALOG_SCHEMA_V2
            && value.get("source_catalog_sha256").and_then(serde_json::Value::as_str)
                == Some(prepared.flat_sha256.as_str())),
        GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1 => Ok(current.catalog_schema_version
            == GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1
            && value == prepared.value),
        _ => Ok(false),
    }
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
        let schema_version = object
            .get("schema_version")
            .and_then(serde_json::Value::as_str)
            .context("flat catalog lacks a schema version")?;
        if schema_version == GOVERNED_DATA_CATALOG_REGISTRY_SCHEMA_V1 {
            continue;
        }
        let id = object
            .get("id")
            .and_then(serde_json::Value::as_str)
            .context("flat catalog lacks an ID")?;
        if !matches!(
            schema_version,
            GOVERNED_DATA_CATALOG_SCHEMA_V1 | GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1
        ) || path.file_stem().and_then(|value| value.to_str()) != Some(id)
            || !ids.insert(id.to_string())
        {
            bail!("flat catalog identity or schema is invalid: {}", path.display());
        }
        if schema_version == GOVERNED_DERIVED_DATA_CATALOG_SCHEMA_V1 {
            let derived: GovernedDerivedDataCatalog = serde_json::from_value(value.clone())
                .with_context(|| format!("parse derived catalog {}", path.display()))?;
            validate_governed_derived_data_catalog(&derived)
                .with_context(|| format!("validate derived catalog {}", path.display()))?;
            verify_derived_catalog_closure(&derived, data_root)?;
        }
        catalogs.push(FlatCatalog {
            id: id.to_string(),
            schema_version: schema_version.to_string(),
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

fn verify_derived_catalog_closure(
    catalog: &GovernedDerivedDataCatalog,
    data_root: &Path,
) -> Result<()> {
    for source in &catalog.sources {
        let relative = source
            .catalog_path
            .strip_prefix("study://data/")
            .context("derived source catalog URI escapes governed data root")?;
        let path = canonical_file(&data_root.join(relative), data_root, "derived source catalog")?;
        if bijux_dna_infra::hash_file_sha256(&path)? != source.catalog_sha256 {
            bail!("derived source catalog digest drifted: {}", path.display());
        }
    }
    for file in &catalog.files {
        let path = canonical_file(Path::new(&file.path), data_root, "derived catalog file")?;
        if fs::metadata(&path)?.len() != file.size_bytes
            || bijux_dna_infra::hash_file_sha256(&path)? != file.sha256
        {
            bail!("derived catalog file identity drifted: {}", path.display());
        }
    }
    Ok(())
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
