//! Owner: bijux-dna-bench-population
//! Population admixture benchmark ownership.
mod model;
pub use model::{LocalVcfAdmixtureSmokeReport, LocalVcfAdmixtureSmokeRow};
pub(crate) mod admission;
pub(crate) mod execution;
pub(crate) mod fixtures;
pub(crate) mod observations;
pub(crate) mod publication;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_admixture_smoke;
