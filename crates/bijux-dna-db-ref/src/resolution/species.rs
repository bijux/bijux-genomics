use anyhow::{anyhow, bail, Result};
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
use bijux_dna_domain_vcf::taxonomy::CoverageRegime;
use std::collections::BTreeSet;

use crate::resolution::resolve_bundle_entry;
use crate::runtime_config::{
    load_toml, workspace_root, AliasesConfig, CoverageRegimesConfig, SpeciesAuthorityConfig,
};
use crate::{
    ContigAliasResolutionReport, ContigAliasResolutionRow, ContigMap, ResolvedSpeciesContext,
    SexChromosomeRule, SexParOrganellarAssetsReport, SpeciesAuthorityEntry, SupportedFeatures,
};

/// # Errors
/// Returns an error if alias config cannot be read or the alias is unknown.
pub fn resolve_species_alias(
    alias: &str,
    requested_build: Option<&str>,
) -> Result<(String, String)> {
    let path = workspace_root().join("configs/runtime/species_aliases.toml");
    let cfg: AliasesConfig = load_toml(&path)?;
    let normalized_alias = alias.trim().to_ascii_lowercase();
    let canonical_species = cfg
        .aliases
        .get(&normalized_alias)
        .cloned()
        .ok_or_else(|| anyhow!("unknown species alias: {alias}"))?;
    let build = requested_build
        .map(str::trim)
        .filter(|build| !build.is_empty())
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
    let known_contigs = known_contig_names(&contig_map);
    for contig in observed_contigs {
        let observed = contig.trim();
        if observed.is_empty() {
            bail!("observed contig name must not be empty");
        }
        let normalized =
            contig_map.aliases.get(observed).cloned().unwrap_or_else(|| observed.to_string());
        if normalized.trim().is_empty() {
            bail!("contig normalization produced empty value for {contig}");
        }
        if !known_contigs.contains(observed) && !known_contigs.contains(normalized.as_str()) {
            bail!("contig {contig} is not declared for {species}:{declared_build}");
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error if alias resolution fails against reference/panel/map compatibility surfaces.
pub fn resolve_contig_aliases_for_assets(
    species: &str,
    build: &str,
    contigs: &[String],
    panel_id: Option<&str>,
    map_id: Option<&str>,
) -> Result<ContigAliasResolutionReport> {
    let bundle = crate::resolution::resolve_reference_bundle(species, build)?;
    let panel = panel_id
        .map(|id| crate::resolution::resolve_panel(species, build, Some(id)))
        .transpose()?;
    let map =
        map_id.map(|id| crate::resolution::resolve_map(species, build, Some(id))).transpose()?;
    if let (Some(panel), Some(map)) = (panel.as_ref(), map.as_ref()) {
        crate::resolution::validate_imputation_tool_compatibility("glimpse", panel, map)?;
    }

    let mut rows = Vec::with_capacity(contigs.len());
    for contig in contigs {
        let normalized = crate::resolution::normalize_contig_name(&bundle, contig)?;
        rows.push(ContigAliasResolutionRow { input: contig.clone(), normalized });
    }

    Ok(ContigAliasResolutionReport {
        schema_version: "bijux.contig_alias_resolution.v1".to_string(),
        species_id: species.to_string(),
        build_id: build.to_string(),
        bundle_id: bundle.bundle_id,
        rows,
        panel_id: panel.map(|entry| entry.id),
        map_id: map.map(|entry| entry.id),
    })
}

/// # Errors
/// Returns an error if sex/PAR or organellar policy entries are missing for the species/build.
pub fn resolve_sex_par_organellar_assets(
    species: &str,
    build: &str,
) -> Result<SexParOrganellarAssetsReport> {
    let sex = resolve_sex_chromosome_rule(species, build)?;
    let organellar = crate::resolution::resolve_organellar_policy(species, build)?;
    let context = resolve_species_context(species, build)?;
    Ok(SexParOrganellarAssetsReport {
        schema_version: "bijux.sex_par_organellar_assets.v1".to_string(),
        species_id: species.to_string(),
        build_id: build.to_string(),
        male_x_ploidy: sex.male_x_ploidy,
        male_y_ploidy: sex.male_y_ploidy,
        par_region_count: sex.par_regions.len(),
        mitochondrion_id: organellar.mitochondrion_id,
        chloroplast_id: organellar.chloroplast_id,
        supported_sex_chr: context.supported_features.sex_chr,
    })
}

fn known_contig_names(contig_map: &ContigMap) -> BTreeSet<&str> {
    let mut names = BTreeSet::new();
    names.insert(contig_map.mitochondrion_id.as_str());
    for (alias, canonical) in &contig_map.aliases {
        names.insert(alias.as_str());
        names.insert(canonical.as_str());
    }
    names
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
    let default_coverage_regime =
        bundle.default_coverage_regime.as_deref().map(parse_coverage_regime).transpose()?;
    let context = SpeciesContext {
        species_id: bundle.species_id.clone(),
        build_id: bundle.build_id.clone(),
        contig_set_digest: bundle.contig_set_digest.clone(),
        contigs: bundle
            .contigs
            .iter()
            .map(|c| ContigSpec { name: c.name.clone(), length_bp: c.length_bp })
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

#[cfg(test)]
mod tests {
    use super::{
        enforce_declared_build_and_contigs, resolve_contig_aliases_for_assets,
        resolve_sex_par_organellar_assets, resolve_species_alias,
    };

    #[test]
    fn resolve_species_alias_trims_alias_and_requested_build() {
        let (species, build) = resolve_species_alias(" Human ", Some(" GRCh38 "))
            .unwrap_or_else(|error| panic!("resolve species alias: {error}"));

        assert_eq!(species, "Homo sapiens");
        assert_eq!(build, "GRCh38");
    }

    #[test]
    fn enforce_declared_build_and_contigs_rejects_unknown_contigs() {
        let Err(error) = enforce_declared_build_and_contigs(
            "Homo sapiens",
            "GRCh38",
            &["not_a_contig".to_string()],
        ) else {
            panic!("unknown contig must fail");
        };

        assert!(error.to_string().contains("not declared"));
    }

    #[test]
    fn resolve_contig_aliases_for_assets_reports_normalized_rows() {
        let report = resolve_contig_aliases_for_assets(
            "Canis lupus",
            "CanFam4",
            &["chr1".to_string(), "chrX".to_string()],
            None,
            None,
        )
        .unwrap_or_else(|error| panic!("resolve contig aliases: {error}"));

        assert_eq!(report.schema_version, "bijux.contig_alias_resolution.v1");
        assert_eq!(report.rows[0].normalized, "1");
        assert_eq!(report.rows[1].normalized, "X");
    }

    #[test]
    fn resolve_sex_par_organellar_assets_exposes_bam_vcf_policy_inputs() {
        let report = resolve_sex_par_organellar_assets("Homo sapiens", "GRCh38")
            .unwrap_or_else(|error| panic!("resolve sex/par/organellar assets: {error}"));

        assert_eq!(report.schema_version, "bijux.sex_par_organellar_assets.v1");
        assert!(report.par_region_count > 0);
        assert_eq!(report.mitochondrion_id, "MT");
        assert!(report.supported_sex_chr);
    }
}
