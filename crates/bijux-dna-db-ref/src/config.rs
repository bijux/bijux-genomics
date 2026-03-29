use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::{
    ContigMap, GeneticMapBankEntry, MapCatalogEntry, MapLockEntry, OrganellarPolicy,
    PanelCatalogEntry, PanelLockEntry, ReferenceBankEntry, ReferenceSet, SexChromosomeRule,
    SpeciesAuthorityEntry,
};

#[derive(Debug, Deserialize)]
pub(crate) struct BundlesConfig {
    #[serde(default)]
    pub(crate) bundle: Vec<BundleEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct BundleEntry {
    pub(crate) bundle_id: String,
    pub(crate) species_id: String,
    pub(crate) build_id: String,
    pub(crate) fasta: String,
    pub(crate) fai: String,
    pub(crate) dict: String,
    pub(crate) contig_set_digest: String,
    #[serde(default)]
    pub(crate) mask_bed: Option<String>,
    #[serde(default)]
    pub(crate) regions_bed: Option<String>,
    pub(crate) source_lock_sha256: String,
    pub(crate) bundle_lock_sha256: String,
    pub(crate) normalization_policy: String,
    #[serde(default)]
    pub(crate) remap: BTreeMap<String, String>,
    pub(crate) sex_system: String,
    pub(crate) par_policy: String,
    #[serde(default)]
    pub(crate) default_coverage_regime: Option<String>,
    #[serde(default)]
    pub(crate) supported_features: SupportedFeatureEntry,
    pub(crate) contigs: Vec<ContigEntry>,
}

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SupportedFeatureEntry {
    #[serde(default)]
    pub(crate) sex_chr: bool,
    #[serde(default)]
    pub(crate) imputation: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ContigEntry {
    pub(crate) name: String,
    pub(crate) length_bp: u64,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AliasesConfig {
    #[serde(default)]
    pub(crate) aliases: BTreeMap<String, String>,
    #[serde(default)]
    pub(crate) default_builds: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct CoverageRegimesConfig {
    #[serde(default)]
    pub(crate) species_profile: BTreeMap<String, String>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct SpeciesAuthorityConfig {
    #[serde(default)]
    pub(crate) species: Vec<SpeciesAuthorityEntry>,
    #[serde(default)]
    pub(crate) contig_map: Vec<ContigMap>,
    #[serde(default)]
    pub(crate) sex_rule: Vec<SexChromosomeRule>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReferenceBankConfig {
    #[serde(default)]
    pub(crate) reference: Vec<ReferenceBankEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct GeneticMapBankConfig {
    #[serde(default)]
    pub(crate) map: Vec<GeneticMapBankEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct OrganellarPolicyConfig {
    #[serde(default)]
    pub(crate) policy: Vec<OrganellarPolicy>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReferenceSetConfig {
    #[serde(default)]
    pub(crate) set: Vec<ReferenceSet>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PanelsConfig {
    #[serde(default)]
    pub(crate) panel: Vec<PanelCatalogEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MapsConfig {
    #[serde(default)]
    pub(crate) map: Vec<MapCatalogEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct PanelLocksConfig {
    #[serde(default)]
    pub(crate) locks: BTreeMap<String, PanelLockEntry>,
}

#[derive(Debug, Deserialize)]
pub(crate) struct MapLocksConfig {
    #[serde(default)]
    pub(crate) locks: BTreeMap<String, MapLockEntry>,
}

pub(crate) fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

pub(crate) fn load_toml<T: for<'a> Deserialize<'a>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    toml::from_str::<T>(&raw).with_context(|| format!("parse {}", path.display()))
}
