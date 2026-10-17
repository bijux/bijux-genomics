# Commands

This file is the SSOT for commands and callable operations owned by
`bijux-dna-stage-contract`.

## Managed Operation Inventory

This crate owns pure Rust contract operations. It does not own CLI commands,
runtime commands, process execution, container execution, or environment
management commands.

### Execution Plan Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `build-execution-plan` | `ExecutionPlan::new` | Construct and validate a deterministic execution plan. |
| `validate-execution-plan` | `lint_execution_plan` | Validate plan identity, stage IO, edge endpoints, artifact bindings, and DAG shape. |
| `validate-execution-plan-strict` | `ExecutionPlan::validate_strict` | Validate a plan against caller-provided stage and tool catalogs. |
| `canonical-execution-plan-json` | `ExecutionPlan::canonical_json` | Project an execution plan into canonical JSON. |
| `hash-execution-plan` | `ExecutionPlan::plan_hash` | Hash canonical execution-plan JSON with SHA-256. |
| `default-stage-edges` | `default_edges_for_stages` | Build deterministic default edges, preferring artifact-bound handoffs. |

### Stage Plan Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `stage-plan-json` | `StagePlanJsonV1::from_plan` | Build the stable JSON projection for a stage plan. |
| `stage-plan-execution-step` | `execution_step_from_stage_plan` | Convert a stage plan into the default execution-step contract. |
| `stage-plan-execution-step-with-id` | `execution_step_from_stage_plan_with_step_id` | Convert a stage plan into an execution step with an explicit step ID. |

### Run Planning Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `build-run-execution-plan` | `build_run_execution_plan` | Assemble a run execution plan from a run spec, tool registry, profile, and run ID. |
| `build-stage-plan` | `build_stage_plan` | Build a stage plan from registry stage/tool contracts and declared IO. |
| `build-tool-execution-spec` | `build_tool_execution_spec` | Project a tool manifest into a tool execution spec. |
| `validate-stage-outputs` | `validate_stage_outputs` | Validate that a run spec and stage spec agree on explicit outputs. |
| `artifact-kind-schema` | `artifact_kind_schema` | Resolve artifact role strings to artifact kind and schema identifiers. |

### Executor Registry Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `list-executor-entries` | `executor_registry::entries` | Return all code-backed executor entries. |
| `has-stage-executor` | `executor_registry::has_executor` | Check whether a stage has a code-backed executor. |
| `stage-executor-entry` | `executor_registry::entry` | Return the executor entry for a stage ID. |

### Binding, Admission And Consumer Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `evaluate-stage-admission` | `evaluate_stage_admission` | Evaluate declared stage/reference admission and preserve typed refusals. |
| `validate-stage-contract` | `validate_stage_contract` | Validate agreement between a stage specification and its tool manifest. |
| `validate-report-parts` | `validate_report_parts` | Validate report identity and declared required payload fields. |
| `canonical-tool-bindings` | `canonical_tool_bindings` | Return the complete validated canonical binding registry. |
| `canonical-retained-tool-bindings` | `canonical_retained_tool_bindings` | Return canonical retained tool bindings. |
| `canonical-external-artifact-tool-bindings` | `canonical_external_artifact_tool_bindings` | Declare restricted external artifact integrations without admitting or executing artifacts. |
| `validate-tool-binding` | `validate_tool_binding` | Validate a binding declaration and its projection/output authorities. |
| `render-tool-command` | `render_tool_command` | Render declared values into argv without execution. |
| `project-tool-parameters` | `project_tool_parameters` | Resolve scientific defaults, input-derived values and operator overrides with resource authority. |
| `validate-effective-tool-parameters` | `validate_effective_tool_parameters` | Validate effective parameter identity, schema and resource consistency. |
| `build-binding-stage-plan` | `build_binding_stage_plan` | Build an immutable stage plan from admitted binding, parameter and artifact authority. |
| `required-binding-output-paths` | `required_binding_output_paths` | Expand exact primary/sidecar paths without filesystem IO. |
| `canonical-downstream-probes` | `canonical_downstream_probes` | Derive canonical tool-independent consumer declarations. |
| `downstream-probes-for-bindings` | `downstream_probes_for_bindings` | Derive consistent consumer declarations for an explicit binding registry. |
| `evaluate-downstream-probe` | `evaluate_downstream_probe` | Evaluate normalized output and preserve failed checks as typed outcomes. |

Registry lookup methods and typed field accessors support these operations; they do not add execution authority.

## Forbidden Command Surfaces

- No Cargo binary targets or `src/bin` command modules.
- No CLI parser ownership.
- No process spawning.
- No runtime command execution.
- No Docker, Apptainer, or environment command ownership.

## Local Verification Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-stage-contract --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --test schemas --no-default-features
```
