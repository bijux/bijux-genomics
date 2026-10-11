//! Public all-domain catalog contracts share one link target and process-state guard.
#[path = "contracts/benchmark/all_domain_readiness/catalog/active_stage_catalog.rs"]
mod active_stage_catalog;
#[path = "contracts/benchmark/all_domain_readiness/catalog/active_stage_catalog_tsv.rs"]
mod active_stage_catalog_tsv;
#[path = "contracts/benchmark/all_domain_readiness/catalog/expected_benchmark_results.rs"]
mod expected_benchmark_results;
#[path = "contracts/benchmark/all_domain_readiness/catalog/expected_benchmark_results_tsv.rs"]
mod expected_benchmark_results_tsv;
#[path = "contracts/benchmark/all_domain_readiness/catalog/local_job_coverage.rs"]
mod local_job_coverage;
#[path = "contracts/benchmark/all_domain_readiness/catalog/local_job_coverage_tsv.rs"]
mod local_job_coverage_tsv;
#[path = "contracts/benchmark/all_domain_readiness/catalog/retained_tools.rs"]
mod retained_tools;
#[path = "contracts/benchmark/all_domain_readiness/catalog/retained_tools_tsv.rs"]
mod retained_tools_tsv;
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;
