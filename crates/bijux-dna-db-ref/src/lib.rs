use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ContigNormalizationPolicy {
    StrictOnly,
    DeterministicRemap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedSpeciesContext {
    pub context: SpeciesContext,
    pub supported_features: SupportedFeatures,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SupportedFeatures {
    pub sex_chr: bool,
    pub imputation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceBundle {
    pub bundle_id: String,
    pub species_id: String,
    pub build_id: String,
    pub fasta: String,
    pub fai: String,
    pub dict: String,
    pub contig_set_digest: String,
    pub mask_bed: Option<String>,
    pub regions_bed: Option<String>,
    pub source_lock_sha256: String,
    pub bundle_lock_sha256: String,
    pub normalization_policy: ContigNormalizationPolicy,
    pub remap_table: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReferenceProvenance {
    pub species_id: String,
    pub build_id: String,
    pub bundle_id: String,
    pub contig_set_digest: String,
    pub source_lock_sha256: String,
    pub bundle_lock_sha256: String,
}

#[derive(Debug, Deserialize)]
struct BundlesConfig {
    #[serde(default)]
    bundle: Vec<BundleEntry>,
}

#[derive(Debug, Deserialize)]
struct BundleEntry {
    bundle_id: String,
    species_id: String,
    build_id: String,
    fasta: String,
    fai: String,
    dict: String,
    contig_set_digest: String,
    #[serde(default)]
    mask_bed: Option<String>,
    #[serde(default)]
    regions_bed: Option<String>,
    source_lock_sha256: String,
    bundle_lock_sha256: String,
    normalization_policy: String,
    #[serde(default)]
    remap: BTreeMap<String, String>,
    sex_system: String,
    par_policy: String,
    #[serde(default)]
    default_coverage_regime: Option<String>,
    #[serde(default)]
    supported_features: SupportedFeatureEntry,
    contigs: Vec<ContigEntry>,
}

#[derive(Debug, Default, Deserialize)]
struct SupportedFeatureEntry {
    #[serde(default)]
    sex_chr: bool,
    #[serde(default)]
    imputation: bool,
}

#[derive(Debug, Deserialize)]
struct ContigEntry {
    name: String,
    length_bp: u64,
}

#[derive(Debug, Deserialize)]
struct AliasesConfig {
    #[serde(default)]
    aliases: BTreeMap<String, String>,
    #[serde(default)]
    default_builds: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
struct CoverageRegimesConfig {
    #[serde(default)]
    species_profile: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PanelCatalogEntry {
    pub id: String,
    pub species_id: String,
    pub build_id: String,
    pub version: String,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
    pub compatibility: CatalogCompatibility,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapCatalogEntry {
    pub id: String,
    pub species_id: String,
    pub build_id: String,
    pub version: String,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
    pub compatibility: MapCompatibility,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CatalogFileEntry {
    pub name: String,
    pub path: String,
    pub format: String,
    pub url: String,
    pub checksum_sha256: String,
    #[serde(default)]
    pub required: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CatalogCompatibility {
    #[serde(default)]
    pub tool_tags: Vec<String>,
    pub requires_phased: bool,
    pub supports_gl_input: bool,
    pub supports_minimac_m3vcf: bool,
    pub glimpse_reference_format: String,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapCompatibility {
    #[serde(default)]
    pub tool_tags: Vec<String>,
    pub coordinate_system: String,
}

#[derive(Debug, Deserialize)]
struct PanelsConfig {
    #[serde(default)]
    panel: Vec<PanelCatalogEntry>,
}

#[derive(Debug, Deserialize)]
struct MapsConfig {
    #[serde(default)]
    map: Vec<MapCatalogEntry>,
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn load_toml<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str::<T>(&raw).with_context(|| format!("parse {}", path.display()))
}

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
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no panel found for {species}:{build}"))
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
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| anyhow!("no map found for {species}:{build}"))
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
    if tool_id == "glimpse"
        && panel
            .compatibility
            .glimpse_reference_format
            .trim()
            .is_empty()
    {
        bail!("GLIMPSE requires declared reference format");
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
mod tests {
    use super::*;

    #[test]
    fn species_context_and_bundle_resolve() {
        let resolved = resolve_species_context("Homo sapiens", "GRCh38")
            .unwrap_or_else(|err| panic!("resolve species context: {err}"));
        assert_eq!(resolved.context.build_id, "GRCh38");
        assert!(resolved.supported_features.imputation);
        let bundle = resolve_reference_bundle("Homo sapiens", "GRCh38")
            .unwrap_or_else(|err| panic!("resolve reference bundle: {err}"));
        assert_eq!(bundle.bundle_id, "hsapiens_grch38_primary");
    }

    #[test]
    fn deterministic_remap_table_is_enforced() {
        let bundle = resolve_reference_bundle("Canis lupus", "CanFam4")
            .unwrap_or_else(|err| panic!("resolve reference bundle: {err}"));
        let mapped = normalize_contig_name(&bundle, "chr1")
            .unwrap_or_else(|err| panic!("normalize contig: {err}"));
        assert_eq!(mapped, "1");
    }

    #[test]
    fn panel_and_map_resolution_work() {
        let panel = resolve_panel("Homo sapiens", "GRCh38", None)
            .unwrap_or_else(|err| panic!("resolve panel: {err}"));
        let map = resolve_map("Homo sapiens", "GRCh38", None)
            .unwrap_or_else(|err| panic!("resolve map: {err}"));
        validate_imputation_tool_compatibility("glimpse", &panel, &map)
            .unwrap_or_else(|err| panic!("compatibility: {err}"));
    }
}
