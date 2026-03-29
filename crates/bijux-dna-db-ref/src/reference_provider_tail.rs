/// # Errors
/// Returns an error if alias config cannot be read or the alias is unknown.
pub fn resolve_species_alias(
    alias: &str,
    requested_build: Option<&str>,
) -> Result<(String, String)> {
    let path = workspace_root().join("configs/runtime/species_aliases.toml");
    let cfg: AliasesConfig = load_toml(&path)?;
    let canonical_species = cfg
        .aliases
        .get(&alias.to_ascii_lowercase())
        .cloned()
        .ok_or_else(|| anyhow!("unknown species alias: {alias}"))?;
    let build = requested_build
        .map(ToString::to_string)
        .or_else(|| cfg.default_builds.get(&canonical_species).cloned())
        .ok_or_else(|| {
            anyhow!("no build provided and no default build for species {canonical_species}")
        })?;
    Ok((canonical_species, build))
}

/// # Errors
/// Returns an error if species authority config cannot be read or species is not declared.
pub fn resolve_species_authority(species: &str) -> Result<SpeciesAuthorityEntry> {
    let path = workspace_root().join("configs/runtime/species.toml");
    let cfg: SpeciesAuthorityConfig = load_toml(&path)?;
    cfg.species
        .into_iter()
        .find(|entry| entry.species_id == species)
        .ok_or_else(|| anyhow!("species authority entry missing for {species}"))
}

/// # Errors
/// Returns an error if contig mapping config cannot be read or mapping entry is missing.
pub fn resolve_contig_map(species: &str, build: &str) -> Result<ContigMap> {
    let path = workspace_root().join("configs/runtime/species.toml");
    let cfg: SpeciesAuthorityConfig = load_toml(&path)?;
    cfg.contig_map
        .into_iter()
        .find(|entry| entry.species_id == species && entry.build_id == build)
        .ok_or_else(|| anyhow!("contig map missing for {species}:{build}"))
}

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
/// Returns an error if sex chromosome rule cannot be loaded.
pub fn resolve_sex_chromosome_rule(species: &str, build: &str) -> Result<SexChromosomeRule> {
    let path = workspace_root().join("configs/runtime/species.toml");
    let cfg: SpeciesAuthorityConfig = load_toml(&path)?;
    cfg.sex_rule
        .into_iter()
        .find(|entry| entry.species_id == species && entry.build_id == build)
        .ok_or_else(|| anyhow!("sex chromosome rule missing for {species}:{build}"))
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
/// Returns an error when declared build/contigs are incompatible with authority metadata.
pub fn enforce_declared_build_and_contigs(
    species: &str,
    declared_build: &str,
    observed_contigs: &[String],
) -> Result<()> {
    let authority = resolve_species_authority(species)?;
    if authority.default_build_id != declared_build {
        bail!(
            "declared build mismatch for {species}: declared={declared_build}, authority_default={}",
            authority.default_build_id
        );
    }
    let contig_map = resolve_contig_map(species, declared_build)?;
    for contig in observed_contigs {
        let normalized = contig_map
            .aliases
            .get(contig)
            .cloned()
            .unwrap_or_else(|| contig.clone());
        if normalized.trim().is_empty() {
            bail!("contig normalization produced empty value for {contig}");
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error if coverage profile config cannot be read.
pub fn resolve_coverage_profile(species: &str, build: &str) -> Result<Option<String>> {
    let path = workspace_root().join("configs/runtime/coverage_regimes.toml");
    let cfg: CoverageRegimesConfig = load_toml(&path)?;
    let key = format!("{species}:{build}");
    Ok(cfg.species_profile.get(&key).cloned())
}

/// # Errors
/// Returns an error when the species/build bundle cannot be resolved.
pub fn resolve_species_context(species: &str, build: &str) -> Result<ResolvedSpeciesContext> {
    let bundle = resolve_bundle_entry(species, build)?;
    let default_coverage_regime = bundle
        .default_coverage_regime
        .as_deref()
        .map(parse_coverage_regime)
        .transpose()?;
    let context = SpeciesContext {
        species_id: bundle.species_id.clone(),
        build_id: bundle.build_id.clone(),
        contig_set_digest: bundle.contig_set_digest.clone(),
        contigs: bundle
            .contigs
            .iter()
            .map(|c| ContigSpec {
                name: c.name.clone(),
                length_bp: c.length_bp,
            })
            .collect(),
        sex_system: bundle.sex_system.clone(),
        par_policy: bundle.par_policy.clone(),
        default_coverage_regime,
    };
    Ok(ResolvedSpeciesContext {
        context,
        supported_features: SupportedFeatures {
            sex_chr: bundle.supported_features.sex_chr,
            imputation: bundle.supported_features.imputation,
        },
    })
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
                if bundle.remap_table.values().any(|v| v == contig) {
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
        .filter(|p| p.species_id == species && p.build_id == build)
        .collect::<Vec<_>>();
    if let Some(id) = panel_id {
        candidates.retain(|p| p.id == id);
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

fn parse_lock_ref(lock_ref: &str) -> Result<(&str, &str)> {
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
        bail!(
            "panel lock entry does not match panel identity {}",
            panel.id
        );
    }
    if entry.files.is_empty() {
        bail!("panel lock entry {} has no files", panel.id);
    }
    for file in &entry.files {
        validate_sha256(&file.checksum_sha256, "panel lock checksum_sha256")?;
    }
    Ok(entry)
}

/// # Errors
/// Returns an error if map lock metadata is missing or malformed.
pub fn resolve_map_lock(map: &MapCatalogEntry) -> Result<MapLockEntry> {
    let (lock_path, key) = parse_lock_ref(&map.lock_ref)?;
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
        validate_sha256(&file.checksum_sha256, "map lock checksum_sha256")?;
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

fn resolve_bundle_entry(species: &str, build: &str) -> Result<BundleEntry> {
    let path = workspace_root().join("configs/runtime/reference_bundles.toml");
    let cfg: BundlesConfig = load_toml(&path)?;
    cfg.bundle
        .into_iter()
        .find(|entry| entry.species_id == species && entry.build_id == build)
        .ok_or_else(|| anyhow!("no reference bundle found for {species}:{build}"))
}

fn validate_sha256(value: &str, name: &str) -> Result<()> {
    let ascii_hex = value.chars().all(|c| c.is_ascii_hexdigit());
    if value.len() != 64 || !ascii_hex {
        bail!("{name} must be 64-char lowercase hex");
    }
    Ok(())
}

fn parse_coverage_regime(raw: &str) -> Result<bijux_dna_domain_vcf::taxonomy::CoverageRegime> {
    match raw {
        "lowcov_gl" => Ok(bijux_dna_domain_vcf::taxonomy::CoverageRegime::LowCovGl),
        "pseudohaploid" => Ok(bijux_dna_domain_vcf::taxonomy::CoverageRegime::Pseudohaploid),
        "diploid" => Ok(bijux_dna_domain_vcf::taxonomy::CoverageRegime::Diploid),
        _ => bail!("unknown coverage regime value: {raw}"),
    }
}

#[cfg(test)]
mod reference_provider_contract {
    use super::*;
    include!("reference_provider_contract.rs");
}
