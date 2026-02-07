# bijux-planner-fastq

## What this crate does
FASTQ planner: selects tools and generates graphs + explain payloads.

## What it must not do (boundaries)
No parsing or execution.

## Role in the stack
Upstream: pipelines. Downstream: engine.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/PLANNER_MODEL.md`, `docs/TOOL_SELECTION.md`, `docs/EXPLAIN_OUTPUT.md`, `docs/STAGE_MAPPING.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Plan JSON and explain payload snapshots.

## Effects & determinism guarantees
Pure planning; deterministic ordering/hashes. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/determinism.rs`, `tests/graph_snapshots.rs`, `tests/explainability.rs`, `tests/plan_snapshots.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
