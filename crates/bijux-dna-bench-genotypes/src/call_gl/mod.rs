//! Call gl benchmark boundary.
pub(crate) mod model;
pub use bijux_dna_bench_model::BenchmarkLocalVcfCallGlSmokeReport as LocalVcfCallGlSmokeReport;
pub(crate) mod observations;
pub(crate) mod workflow;
pub use workflow::run_local_vcf_call_gl_smoke;
pub(crate) mod publication;
#[cfg(test)]
#[path = "../../tests/contracts/call_gl.rs"]
mod tests;
