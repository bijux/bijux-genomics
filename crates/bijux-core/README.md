# bijux-core

## What this crate does
Contract SSOT that everything else builds on: IDs, ExecutionGraph, RunManifest, metrics registry, and canonical serialization.

## What it must not do (boundaries)
No tool selection, no command assembly, no filesystem effects beyond pure serialization helpers.

## Role in the stack
Upstream: none. Downstream: runtime, engine, planners, stages, analyze, benchmarks.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/CONTRACTS.md`, `docs/PUBLIC_API.md`, `docs/INVARIANTS.md`, `docs/SERIALIZATION.md`, `docs/SSOT.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Contract JSON shapes and canonical bytes; no runtime artifacts.

## Effects & determinism guarantees
Pure logic only; any effectful behavior must be in runtime/runner. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/execution_graph_validate.rs`, `tests/canonicalization.rs`, `tests/public_api_lock.rs`, `tests/public_module_tree.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
