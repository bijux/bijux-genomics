//! Owner: bijux-dna-bench-population
//! Population demography benchmark ownership.
mod model;
pub use model::{
    LocalVcfDemographyEstimate, LocalVcfDemographyInsufficientProbe, LocalVcfDemographySmokeReport,
};
pub(crate) mod admission;
pub(crate) mod execution;
pub(crate) mod fixtures;
pub(crate) mod observations;
pub(crate) mod publication;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_demography_smoke;
