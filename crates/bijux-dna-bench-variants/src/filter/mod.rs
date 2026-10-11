//! Filter benchmark services.
pub(crate) mod model;
pub use model::{LocalVcfFilterSmokeMetrics, LocalVcfFilterSmokeReport};
pub(crate) mod admission;
pub(crate) mod execution;
pub(crate) mod fixtures;
pub(crate) mod observations;
pub(crate) mod publication;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_filter_smoke;
