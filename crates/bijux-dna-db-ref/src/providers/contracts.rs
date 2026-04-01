use anyhow::Result;

use crate::{
    ContigMap, GeneticMapBankEntry, MapCatalogEntry, OrganellarPolicy, PanelCatalogEntry,
    ReferenceBankEntry, ReferenceBundle, ReferenceSet, SexChromosomeRule, SpeciesAuthorityEntry,
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
