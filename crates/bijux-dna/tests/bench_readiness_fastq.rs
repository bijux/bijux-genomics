// Standard-harness cases share the same process environment and working-directory guard.
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[path = "benchmark/fastq_readiness/mod.rs"]
mod fastq;
