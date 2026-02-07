# PUBLIC_API

This crate is intentionally tiny. The only public surface should be:

- `execution_plan` (ExecutionPlan, PlanEdge, PlannerContractV1)
- `stage_plan` (StagePlanV1, PlanDecisionReason)
- `stage_plugin` (StageInvocationV1, StagePluginOutputV1)
- `plan_run` (RunExecutionPlan, Executor, DryRunExecutor, build_run_execution_plan)
- `execution_step_from_stage_plan`
- `ArtifactRef`, `StageIO`
