//! Call pseudohaploid benchmark boundary.
pub(crate) mod model;
pub use model::LocalVcfCallPseudohaploidSmokeMetrics;
pub use model::LocalVcfCallPseudohaploidSmokeReport;
pub(crate) mod observations;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_call_pseudohaploid_smoke;
pub(crate) mod publication;
#[cfg(test)]
#[path = "../../tests/contracts/call_pseudohaploid.rs"]
mod tests;
