use anyhow::{anyhow, bail, Result};

use crate::resolution::validate_sha256;
use crate::runtime_config::{load_toml, workspace_root, MapLocksConfig, MapsConfig};
use crate::{MapCatalogEntry, MapLockEntry};

/// # Errors
/// Returns an error if map resolution fails.
pub fn resolve_map(species: &str, build: &str, map_id: Option<&str>) -> Result<MapCatalogEntry> {
    let path = workspace_root().join("configs/vcf/maps/maps.toml");
    let cfg: MapsConfig = load_toml(&path)?;
    let mut candidates = cfg
        .map
        .into_iter()
        .filter(|map| map.species_id == species && map.build_id == build)
        .collect::<Vec<_>>();
    if let Some(id) = map_id {
        candidates.retain(|map| map.id == id);
    }
    let map = candidates
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no map found for {species}:{build}"))?;
    if map.lock_ref.trim().is_empty() {
        bail!("map {} missing required lock_ref metadata", map.id);
    }
    validate_map_catalog_files(&map)?;
    let _ = resolve_map_lock(&map)?;
    Ok(map)
}

fn validate_map_catalog_files(map: &MapCatalogEntry) -> Result<()> {
    if map.files.is_empty() {
        bail!("map {} has no catalog files", map.id);
    }
    for file in &map.files {
        validate_sha256(&file.checksum_sha256, "map catalog checksum_sha256")?;
    }
    Ok(())
}

/// # Errors
/// Returns an error if map lock metadata is missing or malformed.
pub fn resolve_map_lock(map: &MapCatalogEntry) -> Result<MapLockEntry> {
    let (lock_path, key) = crate::resolution::parse_lock_ref(&map.lock_ref)?;
    let path = workspace_root().join("configs/vcf/maps").join(lock_path);
    let cfg: MapLocksConfig = load_toml(&path)?;
    let entry = cfg
        .locks
        .get(key)
        .ok_or_else(|| anyhow!("map lock entry `{key}` not found in {}", path.display()))?
        .clone();
    if entry.map_id != map.id
        || entry.species_id != map.species_id
        || entry.build_id != map.build_id
    {
        bail!("map lock entry does not match map identity {}", map.id);
    }
    if entry.files.is_empty() {
        bail!("map lock entry {} has no files", map.id);
    }
    for file in &entry.files {
        crate::resolution::validate_sha256(&file.checksum_sha256, "map lock checksum_sha256")?;
    }
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::validate_map_catalog_files;
    use crate::{CatalogFileEntry, MapCatalogEntry, MapCompatibility};

    #[test]
    fn validate_map_catalog_files_rejects_bad_checksums() {
        let map = MapCatalogEntry {
            id: "map".to_string(),
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            status: "production".to_string(),
            version: "1.0.0".to_string(),
            lock_ref: "locks/map_locks.toml#locks.map".to_string(),
            citation: None,
            files: vec![CatalogFileEntry {
                name: "recombination_map_tsv".to_string(),
                path: "map.tsv.gz".to_string(),
                format: "tsv.gz".to_string(),
                url: "https://example.org/map.tsv.gz".to_string(),
                checksum_sha256: "BAD".to_string(),
                required: true,
            }],
            compatibility: MapCompatibility {
                tool_tags: vec!["glimpse".to_string()],
                coordinate_system: "bp".to_string(),
            },
        };

        let Err(error) = validate_map_catalog_files(&map) else {
            panic!("bad map checksum must fail");
        };

        assert!(error.to_string().contains("map catalog checksum"));
    }
}
