# bijux-runtime

## What this crate does
Runtime contracts + recording: RunLayout, RunManifest, RunRecord, Provenance, events.

## What it must not do (boundaries)
No tool execution; only writes under run layout.

## Role in the stack
Upstream: engine/runner. Downstream: analyze/benchmark.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/RUNTIME_CONTRACT.md`, `docs/EVENTS.md`, `docs/BOUNDARY.md`, `docs/GLOSSARY.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Run manifests, records, provenance, layout tree.

## Effects & determinism guarantees
Filesystem writes under run layout only. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/reference_example.rs`, `tests/runtime_schema_snapshots.rs`, `tests/manifest_integrity.rs`, `tests/run_layout_contract.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
