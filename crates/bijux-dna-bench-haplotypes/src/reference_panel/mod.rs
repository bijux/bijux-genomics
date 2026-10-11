//! Reference panel benchmark ownership.
pub(crate) mod model;
pub use model::{
    LocalVcfPrepareReferencePanelSmokeMetrics, LocalVcfPrepareReferencePanelSmokeReport,
};
pub(crate) mod admission;
pub(crate) mod execution;
pub(crate) mod fixtures;
pub(crate) mod observations;
pub(crate) mod publication;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_prepare_reference_panel_smoke;
