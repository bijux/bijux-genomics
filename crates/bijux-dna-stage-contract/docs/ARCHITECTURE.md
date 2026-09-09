# Architecture

`bijux-dna-stage-contract` is a pure contract crate. Its architecture keeps
planning payloads separate from execution effects and keeps executor readiness
metadata explicit.

## Source Tree

```text
src/
├── downstream_probe/
│   ├── evaluation.rs       # normalized semantic consumer evaluation
│   ├── mod.rs              # downstream-probe public module surface
│   ├── model.rs            # probe declarations and evaluation outcomes
│   └── registry.rs         # one canonical consumer per semantic stage
├── execution_plan/
│   ├── edge.rs             # artifact-bound plan edges
│   ├── mod.rs              # execution-plan public module surface
│   ├── model.rs            # execution-plan data model
│   ├── support.rs          # canonicalization and hashing support
│   └── validation.rs       # execution-plan structural validation
├── executor_registry/
│   ├── catalog/
│   │   ├── executors.rs    # executor labels and metadata entries
│   │   └── mod.rs          # catalog module surface
│   ├── lookup.rs           # registry lookup helpers
│   ├── mod.rs              # executor-registry public module surface
│   └── types.rs            # executor vocabulary and readiness types
├── plan_run/
│   ├── artifact_catalog.rs # artifact schema mapping
│   ├── mod.rs              # run-plan public module surface
│   ├── planner_contract.rs # planner-facing projections
│   └── stage_builder.rs    # run-plan stage assembly
├── stage_plan/
│   ├── contract.rs         # stage-plan contract model
│   ├── execution_step.rs   # execution-step projections
│   ├── json.rs             # JSON projections
│   ├── mod.rs              # stage-plan public module surface
│   └── reason.rs           # decision reason vocabulary
├── tool_binding/
│   ├── mod.rs              # tool-binding public module surface
│   ├── model.rs            # binding interface and serialized model
│   ├── output_closure.rs   # shared output declaration admission and exact native paths
│   ├── registry.rs         # deterministic stage/tool registry
│   ├── rendering.rs        # pure generic argv projection
│   └── validation.rs       # binding conformance validation
├── lib.rs                  # crate public exports
└── stage_plugin.rs         # stage-plugin invocation and output contracts
```

## Data flow

Planner crates build `StagePlanV1` values, assemble them into an
`ExecutionPlan`, and hand contract payloads downstream. Runtime and runner crates
interpret those payloads; this crate does not execute them.

## Minimality Rules

- Keep the crate root limited to `Cargo.toml`, `README.md`, `docs/`, `src/`,
  and `tests/`.
- Keep source modules scoped to the seven source areas listed above.
- Add new modules only when they express stable contract vocabulary that cannot
  live inside an existing source area. Tool bindings are a separate stable
  concern because generic planners consume them independently of stage-plugin
  result parsing. Downstream probes are separate because one tool-independent
  semantic consumer must close every implementation of a stage.
- Update boundary tests and docs in the same change as any intentional layout
  change.

Output discovery has one declaration validator shared by registry admission and exact-path expansion. Expansion retains native path bytes and rejects paths without file names or collisions between primary and sidecar outputs. Benchmark lifecycle and container qualification consume this contract; it performs no filesystem IO or scientific qualification.

Binding transport objects reject undeclared fields at the binding, input, parameter, output, discovery, sidecar and executable-source boundaries. Legacy omitted input-source fields remain readable as `LegacyUnspecified`; new planning still rejects that source. Semantic admission and native downstream compatibility are separate required contracts.
