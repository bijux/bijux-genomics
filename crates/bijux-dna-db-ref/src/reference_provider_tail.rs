/// # Errors
/// Returns an error if map resolution fails.
pub fn resolve_map(species: &str, build: &str, map_id: Option<&str>) -> Result<MapCatalogEntry> {
    let path = workspace_root().join("configs/vcf/maps/maps.toml");
    let cfg: MapsConfig = load_toml(&path)?;
    let mut candidates = cfg
        .map
        .into_iter()
        .filter(|m| m.species_id == species && m.build_id == build)
        .collect::<Vec<_>>();
    if let Some(id) = map_id {
        candidates.retain(|m| m.id == id);
    }
    let map = candidates
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no map found for {species}:{build}"))?;
    if map.lock_ref.trim().is_empty() {
        bail!("map {} missing required lock_ref metadata", map.id);
    }
    let _ = resolve_map_lock(&map)?;
    Ok(map)
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

/// # Errors
/// Returns an error if tool compatibility requirements are not satisfied.
pub fn validate_imputation_tool_compatibility(
    tool_id: &str,
    panel: &PanelCatalogEntry,
    map: &MapCatalogEntry,
) -> Result<()> {
    if !panel.compatibility.tool_tags.iter().any(|x| x == tool_id) {
        bail!("panel {} not compatible with tool {}", panel.id, tool_id);
    }
    if !map.compatibility.tool_tags.iter().any(|x| x == tool_id) {
        bail!("map {} not compatible with tool {}", map.id, tool_id);
    }
    if tool_id == "minimac4" && !panel.compatibility.supports_minimac_m3vcf {
        bail!("minimac4 requires m3vcf-compatible panel representation");
    }
    if tool_id == "minimac4" && !panel.files.iter().any(|f| f.name == "panel_m3vcf") {
        bail!("minimac4 requires `panel_m3vcf` in panel files");
    }
    if tool_id == "glimpse"
        && panel
            .compatibility
            .glimpse_reference_format
            .trim()
            .is_empty()
    {
        bail!("GLIMPSE requires declared reference format");
    }
    if tool_id == "glimpse"
        && !matches!(
            panel.compatibility.glimpse_reference_format.as_str(),
            "bcf+sites" | "bcf" | "sites"
        )
    {
        bail!("GLIMPSE requires supported reference format (bcf+sites|bcf|sites)");
    }
    if matches!(tool_id, "impute5" | "minimac4") && map.compatibility.coordinate_system != "bp" {
        bail!("{tool_id} requires bp coordinate-system genetic map");
    }
    if tool_id == "impute5" && !panel.compatibility.requires_phased {
        bail!("impute5 requires phased panel compatibility");
    }
    if tool_id == "beagle" && !panel.compatibility.supports_gl_input {
        bail!("beagle requires panel compatibility with GL input");
    }
    Ok(())
}

#[cfg(test)]
mod reference_provider_contract {
    use super::*;
    include!("reference_provider_contract.rs");
}
