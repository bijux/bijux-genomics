//! Call diploid benchmark boundary.
pub(crate) mod model;
pub use model::LocalVcfCallDiploidSmokeMetrics;
pub use model::LocalVcfCallDiploidSmokeReport;
pub(crate) mod observations;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_call_diploid_smoke;
pub(crate) mod publication;
#[cfg(test)]
#[path = "tests.rs"]
mod tests;
