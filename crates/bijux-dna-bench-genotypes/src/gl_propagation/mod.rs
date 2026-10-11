//! Gl propagation benchmark boundary.
pub(crate) mod model;
pub use model::LocalVcfGlPropagationSmokeMetrics;
pub use model::LocalVcfGlPropagationSmokeReport;
pub(crate) mod admission;
pub(crate) mod fixtures;
pub(crate) mod observations;
pub(crate) mod publication;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_gl_propagation_smoke;

pub(crate) mod execution;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
