use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, bail, Context, Result};
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

mod models;

pub use models::{
    BuildId, ContigMap, ContigNormalizationPolicy, GeneticMapBankEntry, OrganellarPolicy,
    ParRegion, ReferenceBankEntry, ReferenceBundle, ReferenceProvenance, ReferenceSet,
    ResolvedSpeciesContext, SexChromosomeRule, SpeciesAuthorityEntry, SupportedFeatures,
};

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

#[derive(Debug, Deserialize)]
struct SpeciesAuthorityConfig {
    #[serde(default)]
    species: Vec<SpeciesAuthorityEntry>,
    #[serde(default)]
    contig_map: Vec<ContigMap>,
    #[serde(default)]
    sex_rule: Vec<SexChromosomeRule>,
}

#[derive(Debug, Deserialize)]
struct ReferenceBankConfig {
    #[serde(default)]
    reference: Vec<ReferenceBankEntry>,
}

#[derive(Debug, Deserialize)]
struct GeneticMapBankConfig {
    #[serde(default)]
    map: Vec<GeneticMapBankEntry>,
}

#[derive(Debug, Deserialize)]
struct OrganellarPolicyConfig {
    #[serde(default)]
    policy: Vec<OrganellarPolicy>,
}

#[derive(Debug, Deserialize)]
struct ReferenceSetConfig {
    #[serde(default)]
    set: Vec<ReferenceSet>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PanelCatalogEntry {
    pub id: String,
    pub species_id: String,
    pub build_id: String,
    #[serde(default)]
    pub status: String,
    pub version: String,
    #[serde(default)]
    pub license: String,
    #[serde(default)]
    pub lock_ref: String,
    #[serde(default)]
    pub citation: Option<String>,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
    pub compatibility: CatalogCompatibility,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapCatalogEntry {
    pub id: String,
    pub species_id: String,
    pub build_id: String,
    #[serde(default)]
    pub status: String,
    pub version: String,
    #[serde(default)]
    pub lock_ref: String,
    #[serde(default)]
    pub citation: Option<String>,
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

#[derive(Debug, Deserialize)]
struct PanelLocksConfig {
    #[serde(default)]
    locks: BTreeMap<String, PanelLockEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct PanelLockEntry {
    pub species_id: String,
    pub build_id: String,
    pub panel_id: String,
    pub version: String,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
}

#[derive(Debug, Deserialize)]
struct MapLocksConfig {
    #[serde(default)]
    locks: BTreeMap<String, MapLockEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct MapLockEntry {
    pub species_id: String,
    pub build_id: String,
    pub map_id: String,
    pub version: String,
    #[serde(default)]
    pub files: Vec<CatalogFileEntry>,
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

pub trait RefService: Send + Sync {
    /// # Errors
    /// Returns an error if species/build resolution configuration cannot be loaded.
    fn resolve_coverage_profile(&self, species: &str, build: &str) -> Result<Option<String>>;
    /// # Errors
    /// Returns an error if the species/build reference bundle cannot be resolved.
    fn resolve_reference_bundle(&self, species: &str, build: &str) -> Result<ReferenceBundle>;
    /// # Errors
    /// Returns an error if panel catalogs cannot be loaded or no matching panel is found.
    fn resolve_panel(
        &self,
        species: &str,
        build: &str,
        panel_id: Option<&str>,
    ) -> Result<PanelCatalogEntry>;
    /// # Errors
    /// Returns an error if map catalogs cannot be loaded or no matching map is found.
    fn resolve_map(
        &self,
        species: &str,
        build: &str,
        map_id: Option<&str>,
    ) -> Result<MapCatalogEntry>;
}

#[allow(clippy::missing_errors_doc)]
pub trait ReferenceProvider: Send + Sync {
    fn resolve_species_authority(&self, species: &str) -> Result<SpeciesAuthorityEntry>;
    fn resolve_reference_bank(&self, species: &str, build: &str) -> Result<ReferenceBankEntry>;
    fn resolve_contig_map(&self, species: &str, build: &str) -> Result<ContigMap>;
    fn resolve_genetic_map_bank(
        &self,
        species: &str,
        build: &str,
        panel_id: Option<&str>,
    ) -> Result<GeneticMapBankEntry>;
    fn resolve_sex_chromosome_rule(&self, species: &str, build: &str) -> Result<SexChromosomeRule>;
    fn resolve_organellar_policy(&self, species: &str, build: &str) -> Result<OrganellarPolicy>;
    fn resolve_default_reference_set(&self, species: &str, usecase: &str) -> Result<ReferenceSet>;
}

#[allow(clippy::missing_errors_doc)]
pub trait PanelProvider: Send + Sync {
    fn resolve_panel(
        &self,
        species: &str,
        build: &str,
        panel_id: Option<&str>,
    ) -> Result<PanelCatalogEntry>;
}

#[allow(clippy::missing_errors_doc)]
pub trait MapProvider: Send + Sync {
    fn resolve_map(
        &self,
        species: &str,
        build: &str,
        map_id: Option<&str>,
    ) -> Result<MapCatalogEntry>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct RuntimeRefService;

impl RefService for RuntimeRefService {
    fn resolve_coverage_profile(&self, species: &str, build: &str) -> Result<Option<String>> {
        resolve_coverage_profile(species, build)
    }

    fn resolve_reference_bundle(&self, species: &str, build: &str) -> Result<ReferenceBundle> {
        resolve_reference_bundle(species, build)
    }

    fn resolve_panel(
        &self,
        species: &str,
        build: &str,
        panel_id: Option<&str>,
    ) -> Result<PanelCatalogEntry> {
        resolve_panel(species, build, panel_id)
    }

    fn resolve_map(
        &self,
        species: &str,
        build: &str,
        map_id: Option<&str>,
    ) -> Result<MapCatalogEntry> {
        resolve_map(species, build, map_id)
    }
}

impl ReferenceProvider for RuntimeRefService {
    fn resolve_species_authority(&self, species: &str) -> Result<SpeciesAuthorityEntry> {
        resolve_species_authority(species)
    }

    fn resolve_reference_bank(&self, species: &str, build: &str) -> Result<ReferenceBankEntry> {
        resolve_reference_bank(species, build)
    }

    fn resolve_contig_map(&self, species: &str, build: &str) -> Result<ContigMap> {
        resolve_contig_map(species, build)
    }

    fn resolve_genetic_map_bank(
        &self,
        species: &str,
        build: &str,
        panel_id: Option<&str>,
    ) -> Result<GeneticMapBankEntry> {
        resolve_genetic_map_bank(species, build, panel_id)
    }

    fn resolve_sex_chromosome_rule(&self, species: &str, build: &str) -> Result<SexChromosomeRule> {
        resolve_sex_chromosome_rule(species, build)
    }

    fn resolve_organellar_policy(&self, species: &str, build: &str) -> Result<OrganellarPolicy> {
        resolve_organellar_policy(species, build)
    }

    fn resolve_default_reference_set(&self, species: &str, usecase: &str) -> Result<ReferenceSet> {
        resolve_default_reference_set(species, usecase)
    }
}

impl PanelProvider for RuntimeRefService {
    fn resolve_panel(
        &self,
        species: &str,
        build: &str,
        panel_id: Option<&str>,
    ) -> Result<PanelCatalogEntry> {
        resolve_panel(species, build, panel_id)
    }
}

include!("reference_provider_tail.rs");
