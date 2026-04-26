pub mod execution_plan;
pub mod executor_registry;
pub mod plan_run;
pub mod stage_plan;
pub mod stage_plugin;

pub use execution_plan::{
    default_edges_for_stages, lint_execution_plan, ExecutionPlan, PlanEdge, PlanValidationContext,
};
pub use executor_registry::{
    entries, entry, has_executor, ReadinessBadge, StageDomain, StageExecutorEntry,
};
pub use plan_run::{
    artifact_kind_schema, build_run_execution_plan, build_stage_plan, build_tool_execution_spec,
    validate_stage_outputs, DryRunExecutor, Executor, PlannerContractV1, RunExecutionPlan,
};
pub use stage_plan::{
    execution_step_from_stage_plan, execution_step_from_stage_plan_with_step_id,
    PlanDecisionReason, PlanReasonKind, PlannedArtifactV1, StagePlanJsonV1, StagePlanV1,
};
pub use stage_plugin::{
    StageEventHintV1, StageInvocationV1, StagePlugin, StagePluginOutputV1, StageReportPartV1,
};

pub use bijux_dna_core::contract::{ArtifactRef, StageIO};
