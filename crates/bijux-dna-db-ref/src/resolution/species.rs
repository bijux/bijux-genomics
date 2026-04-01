use anyhow::{anyhow, bail, Result};
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;

use crate::resolution::resolve_bundle_entry;
use crate::runtime_config::{
    load_toml, workspace_root, AliasesConfig, CoverageRegimesConfig, SpeciesAuthorityConfig,
};
use crate::{
    ContigMap, ResolvedSpeciesContext, SexChromosomeRule, SpeciesAuthorityEntry, SupportedFeatures,
};

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

fn parse_coverage_regime(raw: &str) -> Result<CoverageRegime> {
    match raw {
        "lowcov_gl" => Ok(CoverageRegime::LowCovGl),
        "pseudohaploid" => Ok(CoverageRegime::Pseudohaploid),
        "diploid" => Ok(CoverageRegime::Diploid),
        _ => bail!("unknown coverage regime value: {raw}"),
    }
}
