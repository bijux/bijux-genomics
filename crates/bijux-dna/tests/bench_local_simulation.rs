//! Public local simulation command contracts share one link target and process-state lock.
#[path = "contracts/local_simulation/all_domain/all_domain_failure_files.rs"]
mod all_domain_failure_files;
#[path = "contracts/local_simulation/all_domain/all_domain_failures.rs"]
mod all_domain_failures;
#[path = "contracts/local_simulation/all_domain/all_domain_run_files.rs"]
mod all_domain_run_files;
#[path = "contracts/local_simulation/all_domain/all_domain_runs.rs"]
mod all_domain_runs;
#[path = "contracts/local_simulation/pipelines/pipeline_run_files.rs"]
mod pipeline_run_files;
#[path = "contracts/local_simulation/pipelines/pipeline_runs.rs"]
mod pipeline_runs;
#[path = "contracts/local_simulation/stages/stage_failure_records.rs"]
mod stage_failure_records;
#[path = "contracts/local_simulation/stages/stage_failures.rs"]
mod stage_failures;
#[path = "contracts/local_simulation/stages/stage_outputs.rs"]
mod stage_outputs;
#[path = "contracts/local_simulation/stages/stage_runs.rs"]
mod stage_runs;
#[path = "contracts/banks/bank_fixtures.rs"]
mod support;
