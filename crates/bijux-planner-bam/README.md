# bijux-planner-bam

## What this crate does
BAM planner: selects tools across pre/core/downstream phases and emits plan + explain payloads.
Tool roster lives in the stage adapters for each phase (see `docs/STAGE_MAPPING.md`).
Selection resolves per-phase tool allowlists into concrete adapters.

## What it must not do (boundaries)
No parsing or execution.

## Role in the stack
Upstream: pipelines. Downstream: engine.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/PLANNER_MODEL.md`, `docs/TOOL_SELECTION.md`, `docs/EXPLAIN_OUTPUT.md`,
`docs/STAGE_MAPPING.md`, `docs/ADD_TOOL.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Plan JSON and explain payload snapshots.

## Effects & determinism guarantees
Pure planning; deterministic ordering/hashes. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/determinism.rs`, `tests/graph.rs`,
`tests/explain.rs`, `tests/plan.rs`.

## Start here in code
`src/lib.rs` → `src/selection/tool_selection.rs` → `src/tool_adapters/bam.rs`

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
