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
- `tool_binding` for registry-driven tool/stage declarations, generic argv
  rendering, and conformance validation.

## Root Re-exports

- Execution-plan constructors, validation helpers, plan edges, and plan hashes.
- Run-plan assembly helpers and planner-contract projections.
- Stage-plan models, JSON projections, plan reasons, and execution-step
  projections.
- Stage plugin invocation, output, report, event-hint, and trait contracts.
- `ArtifactRef` and `StageIO` from `bijux-dna-core` for caller ergonomics.
- Canonical artifact contract types, schemas, catalog, and compatibility functions.
- Tool-binding models, the deterministic registry, renderer, and validator.

## Public Types

- `ExecutionPlan`
- `PlanEdge`
- `PlanValidationContext`
- `RunExecutionPlan`
- `StageAdmissionRequestV1`
- `StageAdmissionOutcomeV1`
- `StageRefusalV1`
- `StagePlanV1`
- `StagePlanJsonV1`
- `PlannedArtifactV1`
- `StageArtifactPromiseV1`
- `StageProvenanceV1`
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
- `ToolBinding`
- `ToolBindingV1`
- `ToolBindingRegistry`
- `canonical_external_artifact_tool_bindings`
- `ToolBindingRenderContext`
- `ToolArtifactSourceV1`
- `BindingInputV1`
- `BindingParameterV1`
- `BindingOutputV1`
- `OutputDiscoveryV1`
- `ArtifactRef`
- `ArtifactContractV1`
- `ArtifactContractCatalogV1`
- `ArtifactCompatibilityReportV1`
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
{"metrics":{},"artifacts":[],"operating_mode":"enforced","report_parts":[],"warnings":[],"invariants":[],"event_hints":[]}
```

### ToolBindingV1

```json
{"schema_version":"bijux.tool_binding.v1","binding_id":"fastq.trim_reads__fastp","stage_id":"fastq.trim_reads","tool_id":"fastp","artifact_source":{"kind":"catalog_image","catalog_tool_id":"fastp"},"capabilities":["paired_or_single_reads"],"inputs":[{"name":"reads","artifact_contract_id":"bijux.fastq.artifact.reads.v1","required":true}],"parameters":[{"name":"threads","required":true}],"outputs":[{"name":"trimmed","artifact_contract_id":"bijux.fastq.artifact.reads.v1","discovery":{"kind":"exact_path"}}],"command_template":["${executable}","--in=${input:reads}","--out","${output:trimmed}"],"resources":{"runtime":"apptainer","mem_gb":4,"tmp_gb":2,"threads":8},"normalizer_id":"fastq.trim_reads.v1","downstream_probe_id":"fastq.reads-readable.v1","limitations":[]}
```

## Snapshots And Fixtures

Public surface snapshots live under `tests/fixtures/public_types/` and are
enforced by `tests/schemas/schema/public_type_snapshots.rs`.

Schema payload snapshots live under `tests/fixtures/stage_contract_schema/` and
are enforced by `tests/schemas/schema/schema_snapshots.rs`.

Canonical stage-contract examples for FASTQ, BAM, and VCF live under
`tests/fixtures/docs/` and are validated by the schema suite so the docs point
to executable typed truth instead of prose-only examples.

## Stability Tiers

- Stable: the public modules, root re-exports, and Public Types documented in this file.
- Experimental: future contract helpers are experimental until listed here and covered by the public type/schema snapshots.
- Internal: planner/runtime support code and any item not exposed through the documented public modules or root re-exports.
