// These command contracts share one process guard when run by the standard harness.
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[path = "benchmark/readiness/mod.rs"]
mod readiness;
