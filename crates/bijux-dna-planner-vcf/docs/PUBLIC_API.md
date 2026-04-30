# Public API

The public API is the root planner surface exported from `src/lib.rs`. Internal modules stay private so callers depend on typed planner inputs, explain output, stage plan output, and graph construction functions rather than implementation modules.

## Public Modules
None.

## Root Exports
- `ChunkPlanSettings`
- `VcfPanelLock`
- `VcfPipelineInputs`
- `RegionChunkPlan`
- `PlannerExplainStage`
- `PlannerExplainV1`
  This includes reference context, panel and cohort contracts, reporting contracts, and decision
  traces for governed production review.
- `PLANNER_VERSION`
- `explain_vcf_plan`
- `plan_vcf_stage_plans`
- `plan_vcf_pipeline`
- `plan_vcf_minimal`

## Stability Rules
- Additions must be documented here and covered by boundary or contract tests.
- Changes to graph topology, stage plan JSON, reference context, tool selection, or explain payload shape require snapshot review.
- Runtime execution, command routing, and output parsing do not belong in this API.
