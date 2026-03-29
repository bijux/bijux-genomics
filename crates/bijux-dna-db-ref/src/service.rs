use std::sync::OnceLock;

use anyhow::Result;

use crate::{
    ContigMap, GeneticMapBankEntry, MapCatalogEntry, OrganellarPolicy, PanelCatalogEntry,
    ReferenceBankEntry, ReferenceBundle, ReferenceSet, SexChromosomeRule, SpeciesAuthorityEntry,
    resolve_contig_map, resolve_coverage_profile, resolve_default_reference_set,
    resolve_genetic_map_bank, resolve_map, resolve_organellar_policy, resolve_panel,
    resolve_reference_bank, resolve_reference_bundle, resolve_sex_chromosome_rule,
    resolve_species_authority,
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

impl MapProvider for RuntimeRefService {
    fn resolve_map(
        &self,
        species: &str,
        build: &str,
        map_id: Option<&str>,
    ) -> Result<MapCatalogEntry> {
        resolve_map(species, build, map_id)
    }
}

#[must_use]
pub fn ref_service() -> &'static dyn RefService {
    static SERVICE: OnceLock<RuntimeRefService> = OnceLock::new();
    SERVICE.get_or_init(RuntimeRefService::default)
}
