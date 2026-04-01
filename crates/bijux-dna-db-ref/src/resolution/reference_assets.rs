use anyhow::{anyhow, bail, Result};

use crate::runtime_config::{
    load_toml, workspace_root, BundlesConfig, GeneticMapBankConfig, OrganellarPolicyConfig,
    ReferenceBankConfig, ReferenceSetConfig,
};
use crate::{
    BundleEntry, ContigNormalizationPolicy, GeneticMapBankEntry, OrganellarPolicy,
    ReferenceBankEntry, ReferenceBundle, ReferenceProvenance, ReferenceSet,
};

/// # Errors
/// Returns an error if reference bank config cannot be read or entry is missing.
pub fn resolve_reference_bank(species: &str, build: &str) -> Result<ReferenceBankEntry> {
    let path = workspace_root().join("configs/runtime/reference_bank.toml");
    let cfg: ReferenceBankConfig = load_toml(&path)?;
    let entry = cfg
        .reference
        .into_iter()
        .find(|row| row.species_id == species && row.build_id == build)
        .ok_or_else(|| anyhow!("reference bank entry missing for {species}:{build}"))?;
    validate_sha256(&entry.fasta_sha256, "reference_bank fasta_sha256")?;
    if entry.license_id.trim().is_empty() || entry.license_url.trim().is_empty() {
        bail!("reference bank entry for {species}:{build} missing license metadata");
    }
    Ok(entry)
}

/// # Errors
/// Returns an error if genetic map bank config cannot be read or no matching map exists.
pub fn resolve_genetic_map_bank(
    species: &str,
    build: &str,
    panel_id: Option<&str>,
) -> Result<GeneticMapBankEntry> {
    let path = workspace_root().join("configs/runtime/genetic_map_bank.toml");
    let cfg: GeneticMapBankConfig = load_toml(&path)?;
    let mut candidates = cfg
        .map
        .into_iter()
        .filter(|entry| entry.species_id == species && entry.build_id == build)
        .collect::<Vec<_>>();
    if let Some(panel) = panel_id {
        candidates.retain(|entry| entry.panel_id.as_deref() == Some(panel));
    }
    let selected = candidates
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("genetic map bank entry missing for {species}:{build}"))?;
    validate_sha256(&selected.map_sha256, "genetic_map_bank map_sha256")?;
    Ok(selected)
}

/// # Errors
/// Returns an error if organellar policy cannot be loaded.
pub fn resolve_organellar_policy(species: &str, build: &str) -> Result<OrganellarPolicy> {
    let path = workspace_root().join("configs/runtime/organellar_policy.toml");
    let cfg: OrganellarPolicyConfig = load_toml(&path)?;
    cfg.policy
        .into_iter()
        .find(|entry| entry.species_id == species && entry.build_id == build)
        .ok_or_else(|| anyhow!("organellar policy missing for {species}:{build}"))
}

/// # Errors
/// Returns an error if default reference set cannot be loaded for species/usecase.
pub fn resolve_default_reference_set(species: &str, usecase: &str) -> Result<ReferenceSet> {
    let path = workspace_root().join("configs/runtime/reference_sets.toml");
    let cfg: ReferenceSetConfig = load_toml(&path)?;
    cfg.set
        .into_iter()
        .find(|entry| entry.species_id == species && entry.usecase == usecase)
        .ok_or_else(|| anyhow!("default reference set missing for {species}:{usecase}"))
}

/// # Errors
/// Returns an error when the reference bundle cannot be resolved or violates contract.
pub fn resolve_reference_bundle(species: &str, build: &str) -> Result<ReferenceBundle> {
    let bundle = resolve_bundle_entry(species, build)?;
    validate_sha256(&bundle.source_lock_sha256, "source_lock_sha256")?;
    validate_sha256(&bundle.bundle_lock_sha256, "bundle_lock_sha256")?;
    let normalization_policy = match bundle.normalization_policy.as_str() {
        "strict_only" => ContigNormalizationPolicy::StrictOnly,
        "deterministic_remap" => ContigNormalizationPolicy::DeterministicRemap,
        other => bail!("unknown normalization policy: {other}"),
    };
    if normalization_policy == ContigNormalizationPolicy::DeterministicRemap
        && bundle.remap.is_empty()
    {
        bail!("deterministic_remap requires non-empty remap table");
    }
    Ok(ReferenceBundle {
        bundle_id: bundle.bundle_id.clone(),
        species_id: bundle.species_id.clone(),
        build_id: bundle.build_id.clone(),
        fasta: bundle.fasta.clone(),
        fai: bundle.fai.clone(),
        dict: bundle.dict.clone(),
        contig_set_digest: bundle.contig_set_digest.clone(),
        mask_bed: bundle.mask_bed.clone(),
        regions_bed: bundle.regions_bed.clone(),
        source_lock_sha256: bundle.source_lock_sha256.clone(),
        bundle_lock_sha256: bundle.bundle_lock_sha256.clone(),
        normalization_policy,
        remap_table: bundle.remap.clone(),
    })
}

/// # Errors
/// Returns an error if policy forbids remapping and contig names differ.
pub fn normalize_contig_name(bundle: &ReferenceBundle, contig: &str) -> Result<String> {
    match bundle.normalization_policy {
        ContigNormalizationPolicy::StrictOnly => Ok(contig.to_string()),
        ContigNormalizationPolicy::DeterministicRemap => bundle
            .remap_table
            .get(contig)
            .cloned()
            .or_else(|| {
                if bundle.remap_table.values().any(|value| value == contig) {
                    Some(contig.to_string())
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow!("contig {contig} not in deterministic remap table")),
    }
}

#[must_use]
pub fn reference_provenance(
    species: &str,
    build: &str,
    bundle: &ReferenceBundle,
) -> ReferenceProvenance {
    ReferenceProvenance {
        species_id: species.to_string(),
        build_id: build.to_string(),
        bundle_id: bundle.bundle_id.clone(),
        contig_set_digest: bundle.contig_set_digest.clone(),
        source_lock_sha256: bundle.source_lock_sha256.clone(),
        bundle_lock_sha256: bundle.bundle_lock_sha256.clone(),
    }
}

pub(crate) fn resolve_bundle_entry(species: &str, build: &str) -> Result<BundleEntry> {
    let path = workspace_root().join("configs/runtime/reference_bundles.toml");
    let cfg: BundlesConfig = load_toml(&path)?;
    cfg.bundle
        .into_iter()
        .find(|entry| entry.species_id == species && entry.build_id == build)
        .ok_or_else(|| anyhow!("no reference bundle found for {species}:{build}"))
}
