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
- `StagePlan`
- `StagePlanV1`
- `StageSpecRef`
- `PluginSpec`
- `StageInvocationV1`
- `StagePluginOutputV1`
- `StageReportPartV1`
- `StageEventHintV1`
- `StageExecutorEntry`
- `ReadinessBadge`
- `StageDomain`

## Schema Examples

### StagePlanV1

```json
{"schema_version":"bijux.stage_plan.v1","stage_id":"fastq.trim_reads"}
```

### ExecutionPlanV1

```json
{"schema_version":"bijux.execution_plan.v1","steps":[],"edges":[]}
```

### StagePluginOutputV1

```json
{"schema_version":"bijux.stage_plugin_output.v1","metrics":{}}
```

## Snapshots And Fixtures

Public surface snapshots live under `tests/fixtures/public_types/` and are
enforced by `tests/schemas/schema/public_type_snapshots.rs`.

Schema payload snapshots live under `tests/fixtures/stage_contract_schema/` and
are enforced by `tests/schemas/schema/schema_snapshots.rs`.
