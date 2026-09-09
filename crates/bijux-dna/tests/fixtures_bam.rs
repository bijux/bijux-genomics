// Each standard-harness suite shares one process environment and working-directory guard.
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[path = "fixtures/bam/mod.rs"]
mod bam;
