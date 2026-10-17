# Public API

This crate exposes contract types only. It does not expose runtime execution,
CLI parsing, process execution, or environment control APIs.

## Public Modules

- `downstream_probe` for tool-independent semantic consumer declarations and
  evaluation outcomes.
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
- Downstream-probe models, the canonical stage registry, and semantic evaluator.

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
- `DownstreamProbe`
- `DownstreamProbeV1`
- `DownstreamProbeAssertionV1`
- `DownstreamProbeCheck`
- `DownstreamProbeRegistry`
- `DownstreamProbeEvaluation`
- `canonical_downstream_probes`
- `downstream_probes_for_bindings`
- `evaluate_downstream_probe`
- `ToolBinding`
- `ToolBindingV1`
- `ToolBindingRegistry`
- `canonical_external_artifact_tool_bindings`
- `canonical_retained_tool_bindings`
- `canonical_tool_bindings`
- `ToolBindingRenderContext`
- `ToolArtifactSourceV1`
- `BindingInputSourceV1`
- `BindingInputV1`
- `BindingParameterV1`
- `BindingOutputV1`
- `OutputDiscoveryV1`
- `OutputSidecarDiscoveryV1`
- `BindingStagePlanRequest`
- `ToolParameterProjectionRequest`
- `EffectiveToolParameterSourceV1`
- `EffectiveToolParameterV1`
- `EffectiveToolParametersV1`
- `ArtifactRef`
- `ArtifactContractV1`
- `ArtifactContractCatalogV1`
- `ArtifactCompatibilityReportV1`
- `StageIO`

## Binding And Consumer Operations

`project_tool_parameters` records parameter precedence and resource authority; `validate_effective_tool_parameters` checks that payload before `build_binding_stage_plan` combines it with typed artifacts and image/executable identity. `required_binding_output_paths` expands exact native output names without filesystem access.

`evaluate_stage_admission` and `validate_stage_contract` check declared stage/tool requirements. `validate_report_parts` validates report payloads, while `evaluate_downstream_probe` returns consumer checks for domain-normalized outputs. Callers own acquisition, execution and publication. The complete managed operation roster is in [COMMANDS.md](COMMANDS.md).

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
{"schema_version":"bijux.tool_binding.v1","binding_id":"fastq.trim_reads__fastp","stage_id":"fastq.trim_reads","tool_id":"fastp","artifact_source":{"kind":"catalog_image","catalog_tool_id":"fastp"},"capabilities":["paired_or_single_reads"],"inputs":[{"name":"reads","artifact_contract_id":"bijux.fastq.artifact.reads.v1","required":true}],"parameters":[{"name":"threads","required":true}],"outputs":[{"name":"trimmed","artifact_contract_id":"bijux.fastq.artifact.reads.v1","discovery":{"kind":"exact_path"}}],"normalized_output_contract_id":"bijux.fastq.artifact.reads.v1","command_template":["${executable}","--in=${input:reads}","--out","${output:trimmed}"],"resources":{"runtime":"apptainer","mem_gb":4,"tmp_gb":2,"threads":8},"normalizer_id":"fastq.trim_reads.v1","downstream_probe_id":"fastq.reads-readable.v1","limitations":[]}
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

## Binding command values

`render_tool_command` projects values without execution. Direct argument tokens and complete
`${inputs:name}` projections retain their supplied bytes. In the canonical `/bin/sh -c`,
`/bin/bash -o pipefail -c` and `/bin/bash -lc` script forms, scalar projections inside
single-quoted literals preserve apostrophes as literal characters, including assignment values
and path suffixes. Values without apostrophes produce the same command vectors. The renderer
does not interpret custom shell grammars or make unquoted or double-quoted custom templates
safe for arbitrary values. Callers retain responsibility for admitting executable and path authority.
