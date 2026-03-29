mod catalog;
mod config;
mod models;
mod resolution;
mod service;

use config::BundleEntry;

pub use catalog::{
    CatalogCompatibility, CatalogFileEntry, MapCatalogEntry, MapCompatibility, MapLockEntry,
    PanelCatalogEntry, PanelLockEntry,
};
pub use models::{
    BuildId, ContigMap, ContigNormalizationPolicy, GeneticMapBankEntry, OrganellarPolicy,
    ParRegion, ReferenceBankEntry, ReferenceBundle, ReferenceProvenance, ReferenceSet,
    ResolvedSpeciesContext, SexChromosomeRule, SpeciesAuthorityEntry, SupportedFeatures,
};
pub use resolution::{
    enforce_declared_build_and_contigs, normalize_contig_name, reference_provenance,
    resolve_contig_map, resolve_coverage_profile, resolve_default_reference_set,
    resolve_genetic_map_bank, resolve_map, resolve_map_lock, resolve_organellar_policy,
    resolve_panel, resolve_panel_lock, resolve_reference_bank, resolve_reference_bundle,
    resolve_sex_chromosome_rule, resolve_species_alias, resolve_species_authority,
    resolve_species_context, validate_imputation_tool_compatibility,
};
pub use service::{
    MapProvider, PanelProvider, RefService, ReferenceProvider, RuntimeRefService, ref_service,
};

#[cfg(test)]
mod reference_provider_contract;
