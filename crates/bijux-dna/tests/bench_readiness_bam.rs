// Cases share the process environment guard; nextest partitions individual contracts.
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[path = "benchmark/bam_readiness/mod.rs"]
mod bam;
