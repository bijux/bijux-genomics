//! Call benchmark boundary.
pub(crate) mod model;
pub use bijux_dna_bench_model::BenchmarkLocalVcfCallSmokeReport as LocalVcfCallSmokeReport;
pub(crate) mod admission;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_call_smoke;
pub(crate) mod observations;
pub(crate) mod publication;
#[cfg(test)]
#[path = "../../tests/contracts/call.rs"]
mod tests;
