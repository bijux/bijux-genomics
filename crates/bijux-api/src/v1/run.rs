//! Execution API for v1.
//!
//! Stability: v1 (stable).

pub use crate::args::{
    DryRunRequest, DryRunResponse, ExecuteRequest, ExecuteResponse, ExecuteRunRequest,
    ExecuteRunResult, PlanRequest, PlanResponse, RunRequest, RunResult, RunStatus,
};
pub use crate::handlers::cross::run_fastq_to_bam_profile;
pub use crate::run::{
    dry_run, execute, execute_and_report, execute_run, plan, plan_only, policy_audit,
    replay_manifest, run_pipeline, status, RunMode,
};
pub use bijux_environment::api::{load_image_catalog, load_platform, RunnerKind};
pub use bijux_infra::RUN_LAYOUT_CONTRACT;
pub use bijux_infra::{atomic_write_bytes, ensure_dir, temp_dir, temp_dir_in, write_bytes};

pub use bijux_core::contract::*;
pub use bijux_core::prelude::{
    run_dir, PathSpec, Profile, RunSpec, StageId, ToolId, ToolRegistry, ToolRole,
};
pub use bijux_runtime::manifests::load_manifests;
pub use bijux_runtime::run::{load_profile, new_run_id, resolve_run_base_dir};
pub use bijux_runtime::FactsRowV1;
pub use bijux_stage_contract::StagePlanV1;
pub use bijux_stage_contract::{execution_step_from_stage_plan, DryRunExecutor, Executor};

pub use bijux_core::contract::ExecutionManifest;
pub use bijux_core::prelude::{CategorizedError, ErrorCategory};
pub use bijux_infra::init_logging;
pub use bijux_runner::primitives::execute_step;
pub use bijux_runner::primitives::{build_tool_execution_spec, replay_run};
