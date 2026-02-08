//! Execution API for v1.
//!
//! Stability: v1 (stable).

pub use crate::api_internal::handlers::cross::run_fastq_to_bam_profile;
pub use crate::request_args::{
    DryRunRequest, DryRunResponse, ExecuteRequest, ExecuteResponse, ExecuteRunRequest,
    ExecuteRunResult, PlanRequest, PlanResponse, RunRequest, RunResult, RunStatus,
};
pub use crate::run::{
    dry_run, execute, execute_and_report, execute_run, plan, plan_only, policy_audit,
    replay_manifest, run_pipeline, status, RunMode,
};
pub use bijux_dna_environment::api::{load_image_catalog, load_platform, RuntimeKind};
pub use bijux_dna_infra::RUN_LAYOUT_CONTRACT;
pub use bijux_dna_infra::{atomic_write_bytes, ensure_dir, temp_dir, temp_dir_in, write_bytes};

pub use bijux_dna_core::contract::*;
pub use bijux_dna_core::prelude::{
    run_dir, PathSpec, Profile, RunSpec, StageId, ToolId, ToolRegistry, ToolRole,
};
pub use bijux_dna_runtime::manifests::load_manifests;
pub use bijux_dna_runtime::run::{load_profile, new_run_id, resolve_run_base_dir};
pub use bijux_dna_runtime::FactsRowV1;
pub use bijux_dna_stage_contract::StagePlanV1;
pub use bijux_dna_stage_contract::{execution_step_from_stage_plan, DryRunExecutor, Executor};

pub use bijux_dna_core::contract::ExecutionManifest;
pub use bijux_dna_core::prelude::{CategorizedError, ErrorCategory};
pub use bijux_dna_infra::init_logging;
pub use bijux_dna_runner::backend::docker::execution_spec::build_tool_execution_spec;
pub use bijux_dna_runner::backend::docker::replay::replay_run;
pub use bijux_dna_runner::execute::execute_step;
