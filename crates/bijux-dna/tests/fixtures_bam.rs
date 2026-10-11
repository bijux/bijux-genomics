// Each standard-harness suite shares one process environment and working-directory guard.
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[path = "contracts/fixtures/bam/mod.rs"]
mod bam;
