//! Owner: bijux-dna-bench-vcf
//! Compatibility surface for corpus-owned panel metadata fixtures.
pub use bijux_dna_bench_corpus::panel_metadata::{
    materialize_local_vcf_panel_fixture, resolve_governed_vcf_panel_asset_contract,
    with_local_vcf_panel_fixture_lock, LocalVcfPanelFixtureReport, ResolvedVcfPanelAssetContract,
    ResolvedVcfPanelDerivationContract, GOVERNED_VCF_MAP_ID, GOVERNED_VCF_PANEL_BUILD_ID,
    GOVERNED_VCF_PANEL_ID, GOVERNED_VCF_PANEL_SPECIES_ID,
};
