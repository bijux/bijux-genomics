# Architecture

`bijux-dna-stage-contract` is a pure contract crate. Its architecture keeps
planning payloads separate from execution effects and keeps executor readiness
metadata explicit.

## Source Tree

- `stage_plan/` owns stage-plan models, JSON projections, decision reasons, and
  execution-step projections.
- `execution_plan/` owns plan models, artifact-bound edges, canonicalization
  helpers, hashing, and validation.
- `executor_registry/` owns executor vocabulary, catalog entries, readiness
  badges, and lookup helpers.
- `plan_run/` owns run-plan assembly, artifact schema mapping, and
  planner-contract views.
- `stage_plugin.rs` owns the focused stage plugin invocation and output
  contracts.

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
