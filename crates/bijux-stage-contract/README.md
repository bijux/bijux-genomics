# bijux-stage-contract

## What this crate does
Minimal planning contract types for stage plans and plugins.

## What this crate is NOT
- Not a runtime crate (no execution artifacts).
- Not a runner crate (no process execution).
- Not a docker/container crate.

## What it must not do (boundaries)
No execution details or runner/env concepts.

## Role in the stack
Upstream: core IDs. Downstream: planners/engine.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/CONTRACT.md`, `docs/PUBLIC_API.md`, `docs/SCHEMAS.md`,
`docs/VERSIONING.md`, `docs/MINIMALITY.md`, `docs/CHANGE_RULES.md`.

## Public types
- `ExecutionPlan`
- `StagePlan`
- `StageSpecRef`
- `PluginSpec`

## Key contracts it owns/consumes
Plan JSON shapes and fixtures.

## Effects & determinism guarantees
Pure contract types; deterministic serialization. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/schema/public_type_snapshots.rs`,
`tests/schema/schema_snapshots.rs`, `tests/guardrails/no_execution_scan.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/lib.rs` and `src/execution_plan.rs`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
