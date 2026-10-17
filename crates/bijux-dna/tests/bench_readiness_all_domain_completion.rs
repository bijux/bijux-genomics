//! All-domain completion evidence and gates share one link target and process-state guard.
#[path = "contracts/benchmark/all_domain_readiness/completion/active_scope.rs"]
mod active_scope;
#[path = "contracts/benchmark/all_domain_readiness/completion/operational_benchmark.rs"]
mod operational_benchmark;
#[path = "contracts/benchmark/all_domain_readiness/completion/parser_collector.rs"]
mod parser_collector;
#[path = "contracts/benchmark/all_domain_readiness/completion/parser_collector_file.rs"]
mod parser_collector_file;
#[path = "contracts/benchmark/all_domain_readiness/completion/rules.rs"]
mod rules;
#[path = "contracts/benchmark/all_domain_readiness/completion/rules_file.rs"]
mod rules_file;
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;
