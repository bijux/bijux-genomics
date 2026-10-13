// Readiness commands and parser refusal evidence use explicit repository test support.
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[path = "contracts/benchmark/readiness/mod.rs"]
mod readiness;

#[path = "contracts/benchmark/readiness/parser_failures/mod.rs"]
mod parser_failures;
