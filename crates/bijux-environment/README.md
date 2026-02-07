# bijux-environment

## What this crate does
Deterministic resolution of tool images and environment specs.

## What it must not do (boundaries)
No tool execution or container runs.

## Role in the stack
Upstream: API/planners. Downstream: runner.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/ENV_REFERENCE.md`, `docs/ENV_MATRIX.md`, `docs/SCHEMAS.md`, `docs/BOUNDARY.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Resolved environment specs and digests only.

## Effects & determinism guarantees
Pure resolution; no network execution. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/reference_matrix.rs`, `tests/schema_snapshots.rs`, `tests/guardrails_runtime.rs`, `tests/docs_reference_matrix.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
