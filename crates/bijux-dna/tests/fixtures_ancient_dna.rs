// Each standard-harness suite shares one process environment and working-directory guard.
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[path = "contracts/fixtures/ancient_dna/mod.rs"]
mod ancient_dna;
