//! Owner: bijux-dna-bench-population
//! Population population structure benchmark ownership.
mod model;
pub use model::{
    ConsumedAdmixtureReport, ConsumedPcaReport, LocalVcfPopulationStructureDistanceSummary,
    LocalVcfPopulationStructureSampleGroup, LocalVcfPopulationStructureSmokeReport,
};
pub(crate) mod admission;
pub(crate) mod execution;
pub(crate) mod observations;
pub(crate) mod publication;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_population_structure_smoke;
