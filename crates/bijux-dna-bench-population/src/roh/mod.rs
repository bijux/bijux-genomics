//! Owner: bijux-dna-bench-population
//! Population roh benchmark ownership.
mod model;
pub use model::{
    LocalVcfRohSmokePerSampleSummary, LocalVcfRohSmokeReport, LocalVcfRohSmokeSegment,
};
pub(crate) mod admission;
pub(crate) mod execution;
pub(crate) mod observations;
pub(crate) mod publication;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_roh_smoke;
