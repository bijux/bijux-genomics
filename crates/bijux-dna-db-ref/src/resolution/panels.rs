use anyhow::{anyhow, bail, Result};

use crate::resolution::{parse_lock_ref, validate_sha256};
use crate::runtime_config::{load_toml, workspace_root, PanelLocksConfig, PanelsConfig};
use crate::{PanelCatalogEntry, PanelLockEntry};

/// # Errors
/// Returns an error if panel resolution fails.
pub fn resolve_panel(
    species: &str,
    build: &str,
    panel_id: Option<&str>,
) -> Result<PanelCatalogEntry> {
    let path = workspace_root().join("configs/vcf/panels/panels.toml");
    let cfg: PanelsConfig = load_toml(&path)?;
    let mut candidates = cfg
        .panel
        .into_iter()
        .filter(|panel| panel.species_id == species && panel.build_id == build)
        .collect::<Vec<_>>();
    if let Some(id) = panel_id {
        candidates.retain(|panel| panel.id == id);
    }
    let panel = candidates
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no panel found for {species}:{build}"))?;
    if panel.license.trim().is_empty() {
        bail!("panel {} missing required license metadata", panel.id);
    }
    if panel.lock_ref.trim().is_empty() {
        bail!("panel {} missing required lock_ref metadata", panel.id);
    }
    validate_panel_catalog_files(&panel)?;
    let _ = resolve_panel_lock(&panel)?;
    Ok(panel)
}

fn validate_panel_catalog_files(panel: &PanelCatalogEntry) -> Result<()> {
    if panel.files.is_empty() {
        bail!("panel {} has no catalog files", panel.id);
    }
    for file in &panel.files {
        validate_sha256(&file.checksum_sha256, "panel catalog checksum_sha256")?;
    }
    Ok(())
}

/// # Errors
/// Returns an error if panel lock metadata is missing or malformed.
pub fn resolve_panel_lock(panel: &PanelCatalogEntry) -> Result<PanelLockEntry> {
    let (lock_path, key) = parse_lock_ref(&panel.lock_ref)?;
    let path = workspace_root().join("configs/vcf/panels").join(lock_path);
    let cfg: PanelLocksConfig = load_toml(&path)?;
    let entry = cfg
        .locks
        .get(key)
        .ok_or_else(|| anyhow!("panel lock entry `{key}` not found in {}", path.display()))?
        .clone();
    if entry.panel_id != panel.id
        || entry.species_id != panel.species_id
        || entry.build_id != panel.build_id
    {
        bail!("panel lock entry does not match panel identity {}", panel.id);
    }
    if entry.files.is_empty() {
        bail!("panel lock entry {} has no files", panel.id);
    }
    for file in &entry.files {
        crate::resolution::validate_sha256(&file.checksum_sha256, "panel lock checksum_sha256")?;
    }
    Ok(entry)
}

#[cfg(test)]
mod tests {
    use super::validate_panel_catalog_files;
    use crate::{CatalogCompatibility, CatalogFileEntry, PanelCatalogEntry};

    #[test]
    fn validate_panel_catalog_files_rejects_bad_checksums() {
        let panel = PanelCatalogEntry {
            id: "panel".to_string(),
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            status: "production".to_string(),
            version: "1.0.0".to_string(),
            license: "CC-BY-4.0".to_string(),
            lock_ref: "locks/panel_locks.toml#locks.panel".to_string(),
            citation: None,
            files: vec![CatalogFileEntry {
                name: "panel_vcf".to_string(),
                path: "panel.vcf.gz".to_string(),
                format: "vcf.gz".to_string(),
                url: "https://example.org/panel.vcf.gz".to_string(),
                checksum_sha256: "BAD".to_string(),
                required: true,
            }],
            compatibility: CatalogCompatibility {
                tool_tags: vec!["glimpse".to_string()],
                requires_phased: true,
                supports_gl_input: true,
                supports_minimac_m3vcf: false,
                glimpse_reference_format: "bcf+sites".to_string(),
            },
        };

        let Err(error) = validate_panel_catalog_files(&panel) else {
            panic!("bad panel checksum must fail");
        };

        assert!(error.to_string().contains("panel catalog checksum"));
    }
}
