//! Execution API for v1.
//!
//! Stability: v1 (stable).

mod entrypoints;
mod operator_failure;
mod request_contracts;
mod runtime_support;
mod stage_assets;

pub use entrypoints::{
    dry_run, execute, execute_and_report, execute_run, plan, plan_only, policy_audit,
    replay_manifest, run_fastq_to_bam_profile, run_pipeline, status, RunMode,
};
pub use operator_failure::{
    classify_operator_failure, CategorizedError, ErrorCategory, ErrorHintV1, HintSeverity,
    OperatorFailureV1,
};
pub use request_contracts::{
    DryRunRequest, DryRunResponse, ExecuteRequest, ExecuteResponse, ExecuteRunRequest,
    ExecuteRunResult, PlanRequest, PlanResponse, RunRequest, RunResult, RunStatus,
};
pub use runtime_support::{
    atomic_write_bytes, build_tool_execution_spec, ensure_dir, execution_step_from_stage_plan,
    init_logging, load_image_catalog, load_manifests, load_platform, load_profile, new_run_id,
    replay_run, resolve_run_base_dir, run_command, run_command_with_context, run_dir, temp_dir,
    temp_dir_in, write_bytes, CommandOutputV1, DryRunExecutor, ExecutionManifest, Executor,
    FactsRowV1, PathSpec, Profile, RunSpec, RuntimeKind, StageId, StagePlanV1, ToolId,
    ToolRegistry, ToolRole, RUN_LAYOUT_CONTRACT,
};
pub use stage_assets::{
    stage_external_asset_requirement, stage_requires_local_assets, StageAssetClass,
    StageExternalAssetRequirement,
};
