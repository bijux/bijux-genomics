//! Phasing benchmark ownership.
pub(crate) mod model;
pub use model::{LocalVcfPhasingSmokeMetrics, LocalVcfPhasingSmokeReport};
pub(crate) mod execution;
pub(crate) mod fixtures;
pub(crate) mod observations;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_phasing_smoke;
pub(crate) mod publication;
