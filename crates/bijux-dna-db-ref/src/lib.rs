use anyhow::{anyhow, bail, Result};
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};

mod catalog;
mod config;
mod models;
mod service;

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
pub use service::{
    MapProvider, PanelProvider, RefService, ReferenceProvider, RuntimeRefService, ref_service,
};

include!("reference_provider_tail.rs");
