//! Owner: bijux-dna-bench
//! Campaign lifecycle planning from governed benchmark declarations.

mod authorities;
mod execution;
mod operator;
mod planning;
mod storage;

pub use authorities::{load_execution_authorities, BenchmarkExecutionAuthorities};
pub use execution::{build_execution_spec, BenchmarkExecutionMaterial};
pub use operator::{
    plan_checked_in_benchmark_campaign, status_checked_in_benchmark_campaign,
    BenchmarkCyclePlanRequest, BenchmarkCycleStatusRequest,
};
pub use planning::{
    plan_stage_campaign_attempts, plan_stage_campaign_retry_attempts, verify_campaign_plan_source,
};
pub use storage::{
    ensure_execution_spec, ensure_stage_campaign_attempts, load_execution_spec,
    recover_stage_campaign_retry_lineage,
};
