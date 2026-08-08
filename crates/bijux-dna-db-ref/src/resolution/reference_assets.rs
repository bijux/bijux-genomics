use std::collections::BTreeSet;

use anyhow::{anyhow, bail, Result};

use crate::resolution::validate_sha256;
use crate::runtime_config::{
    load_json, load_toml, workspace_root, BundlesConfig, GeneticMapBankConfig,
    OrganellarPolicyConfig, ReferenceBankConfig, ReferenceSetConfig,
};
use crate::{
    BundleEntry, ContigNormalizationPolicy, GeneticMapBankEntry, OrganellarPolicy,
    ReferenceBankEntry, ReferenceBundle, ReferenceBundleResolverReport, ReferenceProvenance,
    ReferenceSet, ReferenceSourceLock,
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
    validate_md5(&entry.fasta_md5, "reference_bank fasta_md5")?;
    if entry.fasta_size_bytes == 0 {
        bail!("reference bank entry for {species}:{build} has zero source size");
    }
    if entry.source_accession.trim().is_empty() {
        bail!("reference bank entry for {species}:{build} missing source accession");
    }
    if entry.license_id.trim().is_empty() || entry.license_url.trim().is_empty() {
        bail!("reference bank entry for {species}:{build} missing license metadata");
    }
    Ok(entry)
}

/// Resolve and validate the checked reference-source lock against the source bank.
///
/// # Errors
/// Returns an error when the lock is malformed, unordered, incomplete, duplicated, or drifts from
/// the checked source authority.
pub fn resolve_reference_source_lock() -> Result<ReferenceSourceLock> {
    let root = workspace_root();
    let bank_path = root.join("configs/runtime/reference_bank.toml");
    let lock_path = root.join("configs/runtime/references/locks/lock.json");
    let bank: ReferenceBankConfig = load_toml(&bank_path)?;
    let lock: ReferenceSourceLock = load_json(&lock_path)?;
    if lock.schema_version != 2 || lock.source != "configs/runtime/reference_bank.toml" {
        bail!("reference source lock schema or source authority mismatch");
    }
    validate_git_object_id(&lock.source_evidence_commit)?;
    if lock.references.len() != bank.reference.len() {
        bail!("reference source lock does not close over the reference bank");
    }

    let keys = lock
        .references
        .iter()
        .map(|entry| (entry.species_id.as_str(), entry.build_id.as_str()))
        .collect::<Vec<_>>();
    let sorted = keys.iter().copied().collect::<BTreeSet<_>>().into_iter().collect::<Vec<_>>();
    if keys != sorted {
        bail!("reference source lock identities must be unique and sorted");
    }
    for entry in &lock.references {
        validate_md5(&entry.fasta_md5, "reference source lock fasta_md5")?;
        validate_sha256(&entry.fasta_sha256, "reference source lock fasta_sha256")?;
        if entry.size_bytes == 0
            || !entry.evidence_uri.starts_with("study://finalization/acquisitions/")
            || !entry.evidence_uri.ends_with("/completion.json")
            || entry.evidence_uri.contains("..")
        {
            bail!("reference source lock has invalid size or evidence URI");
        }
        let source = bank
            .reference
            .iter()
            .find(|source| {
                source.species_id == entry.species_id && source.build_id == entry.build_id
            })
            .ok_or_else(|| anyhow!("reference source lock contains an unknown identity"))?;
        if source.source_accession != entry.source_accession
            || source.fasta_url != entry.fasta_url
            || source.fasta_md5 != entry.fasta_md5
            || source.fasta_sha256 != entry.fasta_sha256
            || source.fasta_size_bytes != entry.size_bytes
            || source.license_id != entry.license_id
            || source.license_url != entry.license_url
            || source.required_indexes != entry.required_indexes
        {
            bail!("reference source lock drifts from {}:{}", entry.species_id, entry.build_id);
        }
    }
    Ok(lock)
}

fn validate_git_object_id(value: &str) -> Result<()> {
    if value.len() != 40
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("reference source evidence commit must be lowercase 40-character hexadecimal");
    }
    Ok(())
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
    validate_bundle_digests(&bundle)?;
    validate_bundle_contigs(&bundle)?;
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
        contigs: bundle.contigs.iter().map(|contig| contig.name.clone()).collect(),
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
    let contig = contig.trim();
    if contig.is_empty() {
        bail!("contig name must not be empty");
    }
    match bundle.normalization_policy {
        ContigNormalizationPolicy::StrictOnly => {
            if bundle.contigs.iter().any(|canonical| canonical == contig) {
                return Ok(contig.to_string());
            }
            bail!("contig {contig} not declared for bundle {}", bundle.bundle_id);
        }
        ContigNormalizationPolicy::DeterministicRemap => {
            let normalized = bundle
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
                .ok_or_else(|| anyhow!("contig {contig} not in deterministic remap table"))?;
            if bundle.contigs.iter().any(|canonical| canonical == &normalized) {
                return Ok(normalized);
            }
            bail!(
                "contig {contig} normalized to {normalized}, which is not declared for bundle {}",
                bundle.bundle_id
            );
        }
    }
}

#[must_use]
pub fn reference_provenance(
    _species: &str,
    _build: &str,
    bundle: &ReferenceBundle,
) -> ReferenceProvenance {
    ReferenceProvenance {
        species_id: bundle.species_id.clone(),
        build_id: bundle.build_id.clone(),
        bundle_id: bundle.bundle_id.clone(),
        contig_set_digest: bundle.contig_set_digest.clone(),
        source_lock_sha256: bundle.source_lock_sha256.clone(),
        bundle_lock_sha256: bundle.bundle_lock_sha256.clone(),
    }
}

fn validate_md5(value: &str, label: &str) -> Result<()> {
    if value.len() != 32
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!("{label} must be lowercase 32-character hexadecimal");
    }
    Ok(())
}

/// # Errors
/// Returns an error if any bundle, alias, panel, map, or compatibility contract fails.
pub fn resolve_reference_bundle_contract(
    species: &str,
    build: &str,
    panel_id: Option<&str>,
    map_id: Option<&str>,
    compatibility_tool: Option<&str>,
) -> Result<ReferenceBundleResolverReport> {
    let bundle = resolve_reference_bundle(species, build)?;
    let contig_map = crate::resolution::resolve_contig_map(species, build)?;
    let map_bank = resolve_genetic_map_bank(species, build, panel_id).ok();

    let resolved_panel = panel_id
        .map(|id| crate::resolution::resolve_panel(species, build, Some(id)))
        .transpose()?;
    let resolved_map =
        map_id.map(|id| crate::resolution::resolve_map(species, build, Some(id))).transpose()?;

    if let Some(tool) = compatibility_tool {
        let Some(panel) = resolved_panel.as_ref() else {
            bail!("compatibility check for {tool} requires a resolved panel");
        };
        let Some(map) = resolved_map.as_ref() else {
            bail!("compatibility check for {tool} requires a resolved map");
        };
        crate::resolution::validate_imputation_tool_compatibility(tool, panel, map)?;
    }

    Ok(ReferenceBundleResolverReport {
        schema_version: "bijux.reference_bundle_resolver.v1".to_string(),
        species_id: species.to_string(),
        build_id: build.to_string(),
        bundle_id: bundle.bundle_id,
        contig_aliases: contig_map.aliases,
        panel_id: resolved_panel.as_ref().map(|entry| entry.id.clone()),
        map_id: resolved_map.as_ref().map(|entry| entry.id.clone()),
        map_bank_id: map_bank.map(|entry| entry.id),
        compatibility_checked_tool: compatibility_tool.map(ToString::to_string),
    })
}

pub(crate) fn resolve_bundle_entry(species: &str, build: &str) -> Result<BundleEntry> {
    let path = workspace_root().join("configs/runtime/reference_bundles.toml");
    let cfg: BundlesConfig = load_toml(&path)?;
    cfg.bundle
        .into_iter()
        .find(|entry| entry.species_id == species && entry.build_id == build)
        .ok_or_else(|| anyhow!("no reference bundle found for {species}:{build}"))
}

fn validate_bundle_digests(bundle: &BundleEntry) -> Result<()> {
    validate_sha256(&bundle.contig_set_digest, "contig_set_digest")?;
    validate_sha256(&bundle.source_lock_sha256, "source_lock_sha256")?;
    validate_sha256(&bundle.bundle_lock_sha256, "bundle_lock_sha256")
}

fn validate_bundle_contigs(bundle: &BundleEntry) -> Result<()> {
    if bundle.contigs.is_empty() {
        bail!("reference bundle {} must declare at least one contig", bundle.bundle_id);
    }
    for contig in &bundle.contigs {
        if contig.name.trim().is_empty() {
            bail!("reference bundle {} contains an empty contig name", bundle.bundle_id);
        }
        if contig.length_bp == 0 {
            bail!(
                "reference bundle {} contig {} must have positive length",
                bundle.bundle_id,
                contig.name
            );
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        resolve_reference_bundle_contract, validate_bundle_contigs, validate_bundle_digests,
        validate_md5,
    };
    use crate::runtime_config::{BundleEntry, ContigEntry, SupportedFeatureEntry};
    use std::collections::BTreeMap;

    #[test]
    fn validate_bundle_digests_rejects_invalid_contig_set_digest() {
        let bundle = BundleEntry {
            bundle_id: "bundle".to_string(),
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            fasta: "ref.fa.gz".to_string(),
            fai: "ref.fa.gz.fai".to_string(),
            dict: "ref.dict".to_string(),
            contig_set_digest: "not-a-digest".to_string(),
            mask_bed: None,
            regions_bed: None,
            source_lock_sha256: "a".repeat(64),
            bundle_lock_sha256: "b".repeat(64),
            normalization_policy: "strict_only".to_string(),
            remap: BTreeMap::new(),
            sex_system: "xy".to_string(),
            par_policy: "unsupported".to_string(),
            default_coverage_regime: None,
            supported_features: SupportedFeatureEntry::default(),
            contigs: vec![ContigEntry { name: "1".to_string(), length_bp: 1 }],
        };

        let Err(error) = validate_bundle_digests(&bundle) else {
            panic!("invalid contig digest must fail");
        };

        assert!(error.to_string().contains("contig_set_digest"));
    }

    #[test]
    fn reference_source_md5_rejects_uppercase_and_wrong_length() {
        assert!(validate_md5(&"A".repeat(32), "source md5").is_err());
        assert!(validate_md5(&"a".repeat(31), "source md5").is_err());
        assert!(validate_md5(&"a".repeat(32), "source md5").is_ok());
    }

    #[test]
    fn validate_bundle_contigs_rejects_empty_contig_sets() {
        let bundle = BundleEntry {
            bundle_id: "bundle".to_string(),
            species_id: "Homo sapiens".to_string(),
            build_id: "GRCh38".to_string(),
            fasta: "ref.fa.gz".to_string(),
            fai: "ref.fa.gz.fai".to_string(),
            dict: "ref.dict".to_string(),
            contig_set_digest: "a".repeat(64),
            mask_bed: None,
            regions_bed: None,
            source_lock_sha256: "a".repeat(64),
            bundle_lock_sha256: "b".repeat(64),
            normalization_policy: "strict_only".to_string(),
            remap: BTreeMap::new(),
            sex_system: "xy".to_string(),
            par_policy: "unsupported".to_string(),
            default_coverage_regime: None,
            supported_features: SupportedFeatureEntry::default(),
            contigs: Vec::new(),
        };

        let Err(error) = validate_bundle_contigs(&bundle) else {
            panic!("empty contig set must fail");
        };

        assert!(error.to_string().contains("at least one contig"));
    }

    #[test]
    fn resolve_reference_bundle_preserves_declared_contigs() {
        let bundle = super::resolve_reference_bundle("Homo sapiens", "GRCh38")
            .unwrap_or_else(|error| panic!("resolve reference bundle: {error}"));

        assert!(bundle.contigs.iter().any(|contig| contig == "1"));
        assert!(bundle.contigs.iter().any(|contig| contig == "X"));
    }

    #[test]
    fn strict_contig_normalization_rejects_undeclared_contigs() {
        let bundle = super::resolve_reference_bundle("Homo sapiens", "GRCh38")
            .unwrap_or_else(|error| panic!("resolve reference bundle: {error}"));

        let Err(error) = super::normalize_contig_name(&bundle, "chr1") else {
            panic!("strict bundle must reject non-canonical contig");
        };

        assert!(error.to_string().contains("not declared"));
    }

    #[test]
    fn reference_provenance_uses_resolved_bundle_identity() {
        let bundle = super::resolve_reference_bundle("Homo sapiens", "GRCh38")
            .unwrap_or_else(|error| panic!("resolve reference bundle: {error}"));

        let provenance = super::reference_provenance("wrong species", "wrong build", &bundle);

        assert_eq!(provenance.species_id, bundle.species_id);
        assert_eq!(provenance.build_id, bundle.build_id);
    }

    #[test]
    fn reference_bundle_contract_resolves_aliases_and_optional_compatibility() {
        let report = resolve_reference_bundle_contract(
            "Homo sapiens",
            "GRCh38",
            Some("hsapiens_grch38_mini"),
            Some("hsapiens_grch38_chr_map"),
            Some("glimpse"),
        )
        .unwrap_or_else(|error| panic!("resolve reference bundle contract: {error}"));

        assert_eq!(report.schema_version, "bijux.reference_bundle_resolver.v1");
        assert_eq!(report.panel_id.as_deref(), Some("hsapiens_grch38_mini"));
        assert_eq!(report.map_id.as_deref(), Some("hsapiens_grch38_chr_map"));
        assert!(report.contig_aliases.contains_key("chr1"));
    }

    #[test]
    fn reference_bundle_contract_rejects_tool_check_without_panel_and_map() {
        let Err(error) = resolve_reference_bundle_contract(
            "Homo sapiens",
            "GRCh38",
            None,
            None,
            Some("glimpse"),
        ) else {
            panic!("tool compatibility without panel/map must fail");
        };

        assert!(error.to_string().contains("requires a resolved panel"));
    }
}
