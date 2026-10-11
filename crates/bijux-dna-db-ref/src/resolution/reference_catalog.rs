use std::path::Path;

use anyhow::{ensure, Result};

use crate::runtime_config::{load_toml, BundlesConfig, SpeciesAuthorityConfig};
use crate::{ReferenceBundleIdentity, SpeciesAuthorityEntry};

/// Find a species authority in the explicitly selected repository catalog.
///
/// # Errors
/// Rejects unreadable or malformed catalogs and ambiguous matching species identities.
pub fn find_species_authority_at(
    repo_root: &Path,
    species_id: &str,
) -> Result<Option<SpeciesAuthorityEntry>> {
    let path = repo_root.join("configs/runtime/species.toml");
    let catalog: SpeciesAuthorityConfig = load_toml(&path)?;
    let mut matches = catalog.species.into_iter().filter(|row| row.species_id == species_id);
    let selected = matches.next();
    ensure!(
        matches.next().is_none(),
        "species authority identity `{species_id}` is ambiguous in {}",
        path.display()
    );
    Ok(selected)
}

/// Find a bundle's declared identity using the complete owner configuration schema.
///
/// This metadata lookup does not qualify reference bytes, digests or artifact readiness.
///
/// # Errors
/// Rejects unreadable or malformed catalogs and ambiguous matching bundle identities.
pub fn find_reference_bundle_identity_at(
    repo_root: &Path,
    bundle_id: &str,
) -> Result<Option<ReferenceBundleIdentity>> {
    let path = repo_root.join("configs/runtime/reference_bundles.toml");
    let catalog: BundlesConfig = load_toml(&path)?;
    let mut matches = catalog.bundle.into_iter().filter(|row| row.bundle_id == bundle_id);
    let selected = matches.next();
    ensure!(
        matches.next().is_none(),
        "reference bundle identity `{bundle_id}` is ambiguous in {}",
        path.display()
    );
    Ok(selected.map(|row| ReferenceBundleIdentity {
        bundle_id: row.bundle_id,
        species_id: row.species_id,
        build_id: row.build_id,
    }))
}
