//! Public all-domain admission contracts share one link target and process-state guard.
#[path = "contracts/benchmark/all_domain_readiness/admission/active_scope_blockers.rs"]
mod active_scope_blockers;
#[path = "contracts/benchmark/all_domain_readiness/admission/active_scope_blockers_tsv.rs"]
mod active_scope_blockers_tsv;
#[path = "contracts/benchmark/all_domain_readiness/admission/no_declared_only_rows.rs"]
mod no_declared_only_rows;
#[path = "contracts/benchmark/all_domain_readiness/admission/no_declared_only_rows_file.rs"]
mod no_declared_only_rows_file;
#[path = "contracts/benchmark/all_domain_readiness/admission/no_not_benchmark_ready_rows.rs"]
mod no_not_benchmark_ready_rows;
#[path = "contracts/benchmark/all_domain_readiness/admission/no_not_benchmark_ready_rows_file.rs"]
mod no_not_benchmark_ready_rows_file;
#[path = "contracts/benchmark/all_domain_readiness/admission/no_planned_rows.rs"]
mod no_planned_rows;
#[path = "contracts/benchmark/all_domain_readiness/admission/no_planned_rows_file.rs"]
mod no_planned_rows_file;
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;
