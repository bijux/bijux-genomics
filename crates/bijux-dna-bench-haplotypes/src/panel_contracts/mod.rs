//! Panel contracts benchmark ownership.
pub(crate) mod model;
pub(crate) use model::GovernedVcfPanelWorkflowSmokeContract;
pub(crate) mod admission;
pub(crate) mod publication;
pub(crate) use admission::bindings::resolve_governed_vcf_panel_workflow_smoke_contract;
pub(crate) use bijux_dna_bench_corpus::panel_metadata::governed_vcf_panel_species_context;
pub(crate) use bijux_dna_bench_corpus::panel_metadata::resolve_governed_vcf_panel_asset_contract;
pub(crate) use model::DEFAULT_STAGE_RESULT_NAME;
pub(crate) use publication::stage_result::build_stage_result_manifest;
