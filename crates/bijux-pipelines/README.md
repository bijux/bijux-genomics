# bijux-pipelines

## What this crate does
Scientific pipeline presets and profiles with defaults ledger.

## What it must not do (boundaries)
No execution or tool selection.

## Role in the stack
Upstream: domain contracts. Downstream: planners/analyze.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/PIPELINES.md`, `docs/PIPELINE_MODEL.md`, `docs/DEFAULTS_LEDGER.md`, `docs/PIPELINE_VERSIONING.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Defaults ledger and profile snapshots.

## Effects & determinism guarantees
Pure data only; deterministic ordering. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/pipeline_registry_snapshot.rs`, `tests/pipeline_completeness.rs`, `tests/override_precedence.rs`, `tests/pipeline_ids_unique.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
