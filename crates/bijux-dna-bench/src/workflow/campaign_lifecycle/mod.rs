//! Owner: bijux-dna-bench
//! Campaign lifecycle planning from governed benchmark declarations.

mod authorities;
mod execution;
mod operator;
mod planning;
mod preparation;
mod storage;

pub use authorities::{load_execution_authorities, BenchmarkExecutionAuthorities};
pub use bijux_dna_bench_model::{
    BenchmarkCycleAdmission, BenchmarkCycleAdmissionState, BenchmarkCycleAdmissionUnit,
    BenchmarkCyclePreparation, BenchmarkCyclePreparationState, BenchmarkCyclePreparationUnit,
};
pub use execution::{build_execution_spec, BenchmarkExecutionMaterial};
pub use operator::{
    admit_checked_in_benchmark_campaign, load_checked_in_benchmark_campaign_plan,
    plan_checked_in_benchmark_campaign, status_checked_in_benchmark_campaign,
    BenchmarkCycleAdmissionRequest, BenchmarkCyclePlanRequest, BenchmarkCycleStatusRequest,
};
pub use planning::{
    plan_stage_campaign_attempts, plan_stage_campaign_retry_attempts, verify_campaign_plan_source,
};
pub use preparation::{prepare_checked_in_benchmark_campaign, BenchmarkCyclePreparationRequest};
pub use storage::{
    ensure_execution_spec, ensure_stage_campaign_attempts, load_execution_spec,
    recover_stage_campaign_retry_lineage,
};
