//! Reference-panel metadata fixtures and their authoritative derivation observations.
mod model;
pub use model::{
    LocalVcfPanelFixtureReport, ResolvedVcfPanelAssetContract, ResolvedVcfPanelDerivationContract,
    GOVERNED_VCF_MAP_ID, GOVERNED_VCF_PANEL_BUILD_ID, GOVERNED_VCF_PANEL_ID,
    GOVERNED_VCF_PANEL_SPECIES_ID,
};
mod admission;
pub use admission::resolve_governed_vcf_panel_asset_contract;
mod materialization;
pub use materialization::materialize_local_vcf_panel_fixture;
mod exclusion;
pub use exclusion::with_local_vcf_panel_fixture_lock;
mod marker_admission;
mod markers;
