# Public API

This crate exposes contract types only. It does not expose runtime execution,
CLI parsing, process execution, or environment control APIs.

## Public Modules

- `execution_plan` for deterministic execution-plan models, edges, hashes, and
  validation.
- `executor_registry` for code-backed executor labels and readiness metadata.
- `plan_run` for run-plan assembly and planner contract views.
- `stage_plan` for stage-plan models, JSON projections, decision reasons, and
  execution-step projections.
- `stage_plugin` for stage invocation and parsed-output payload contracts.

## Root Re-exports

- Execution-plan constructors, validation helpers, plan edges, and plan hashes.
- Run-plan assembly helpers and planner-contract projections.
- Stage-plan models, JSON projections, plan reasons, and execution-step
  projections.
- Stage plugin invocation, output, report, event-hint, and trait contracts.
- `ArtifactRef` and `StageIO` from `bijux-dna-core` for caller ergonomics.

## Public Types

- `ExecutionPlan`
- `PlanEdge`
- `PlanValidationContext`
- `RunExecutionPlan`
- `StagePlanV1`
- `StagePlanJsonV1`
- `PlannedArtifactV1`
- `PlannerContractV1`
- `StageInvocationV1`
- `StagePluginOutputV1`
- `StageReportPartV1`
- `StageEventHintV1`
- `StagePlugin`
- `Executor`
- `DryRunExecutor`
- `PlanDecisionReason`
- `PlanReasonKind`
- `StageExecutorEntry`
- `ReadinessBadge`
- `StageDomain`
- `ArtifactRef`
- `StageIO`

## Shape Examples

These examples are abbreviated for readability. Fixture snapshots under
`tests/fixtures/` are the exact serialized contracts.

### StagePlanV1

```json
{"stage_id":"fastq.trim_reads","stage_version":1,"tool_id":"fastp","tool_version":"1.0","command":{"template":["fastp"]}}
```

### ExecutionPlanV1

```json
{"schema_version":"bijux.execution_plan.v1","pipeline_id":"fastq-to-fastq__default__v1","planner_version":"planner","stages":[],"edges":[]}
```

### StagePluginOutputV1

```json
{"metrics":{},"artifacts":[],"report_parts":[],"warnings":[],"invariants":[],"event_hints":[]}
```

## Snapshots And Fixtures

Public surface snapshots live under `tests/fixtures/public_types/` and are
enforced by `tests/schemas/schema/public_type_snapshots.rs`.

Schema payload snapshots live under `tests/fixtures/stage_contract_schema/` and
are enforced by `tests/schemas/schema/schema_snapshots.rs`.

## Stability Tiers

- Stable: the public modules, root re-exports, and Public Types documented in this file.
- Experimental: future contract helpers are experimental until listed here and covered by the public type/schema snapshots.
- Internal: planner/runtime support code and any item not exposed through the documented public modules or root re-exports.
