use anyhow::{Result, anyhow, bail};

use crate::config::{PanelLocksConfig, PanelsConfig, load_toml, workspace_root};
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
    let _ = resolve_panel_lock(&panel)?;
    Ok(panel)
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

pub(crate) fn parse_lock_ref(lock_ref: &str) -> Result<(&str, &str)> {
    let (path, anchor) = lock_ref
        .split_once('#')
        .ok_or_else(|| anyhow!("invalid lock_ref `{lock_ref}`: missing #anchor"))?;
    let key = anchor
        .strip_prefix("locks.")
        .ok_or_else(|| anyhow!("invalid lock_ref `{lock_ref}`: anchor must start with `locks.`"))?;
    if path.trim().is_empty() || key.trim().is_empty() {
        bail!("invalid lock_ref `{lock_ref}`: empty path or key");
    }
    Ok((path, key))
}
