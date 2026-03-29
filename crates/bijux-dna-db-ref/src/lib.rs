use anyhow::{anyhow, bail, Result};
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
use std::sync::OnceLock;

mod catalog;
mod config;
mod models;

use config::{
    AliasesConfig, BundleEntry, BundlesConfig, CoverageRegimesConfig, GeneticMapBankConfig,
    MapLocksConfig, MapsConfig, OrganellarPolicyConfig, PanelLocksConfig, PanelsConfig,
    ReferenceBankConfig, ReferenceSetConfig, SpeciesAuthorityConfig, load_toml, workspace_root,
};

pub use catalog::{
    CatalogCompatibility, CatalogFileEntry, MapCatalogEntry, MapCompatibility, MapLockEntry,
    PanelCatalogEntry, PanelLockEntry,
};
pub use models::{
    BuildId, ContigMap, ContigNormalizationPolicy, GeneticMapBankEntry, OrganellarPolicy,
    ParRegion, ReferenceBankEntry, ReferenceBundle, ReferenceProvenance, ReferenceSet,
    ResolvedSpeciesContext, SexChromosomeRule, SpeciesAuthorityEntry, SupportedFeatures,
};

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
