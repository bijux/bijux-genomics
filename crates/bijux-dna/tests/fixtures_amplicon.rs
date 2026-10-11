// Each standard-harness suite shares one process environment and working-directory guard.
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[path = "contracts/fixtures/amplicon/mod.rs"]
mod amplicon;
