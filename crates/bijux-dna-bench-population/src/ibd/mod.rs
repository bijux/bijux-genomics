//! Owner: bijux-dna-bench-population
//! Population ibd benchmark ownership.
mod model;
pub use bijux_dna_bench_model::BenchmarkLocalVcfIbdSmokeReport as LocalVcfIbdSmokeReport;
pub(crate) mod admission;
pub(crate) mod execution;
pub(crate) mod fixtures;
pub(crate) mod observations;
pub(crate) mod publication;
pub(crate) mod workflow;
pub(crate) use execution::exclusion::benchmark_output_lock;
pub use workflow::run_local_vcf_ibd_smoke;
pub(crate) use workflow::run_local_vcf_ibd_smoke_unlocked;
