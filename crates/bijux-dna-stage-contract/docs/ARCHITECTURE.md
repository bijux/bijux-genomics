# Architecture

`bijux-dna-stage-contract` is a pure contract crate. Its architecture keeps
planning payloads separate from execution effects and keeps executor readiness
metadata explicit.

## Source Tree

```text
src/
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
- Keep source modules scoped to the five source areas listed above.
- Add new modules only when they express stable contract vocabulary that cannot
  live inside an existing source area.
- Update boundary tests and docs in the same change as any intentional layout
  change.
