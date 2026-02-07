# bijux-planner-fastq

## What this crate does
Selects tools and generates FASTQ execution graphs and explainability payloads.

## What it must not do (boundaries)
Must not parse tool outputs or execute commands. Parsing lives in stages; execution in runner.

## Public API / entrypoints
Planner entrypoints are in `src/lib.rs` with contracts documented in `docs/PLANNER_MODEL.md`.

## Key contracts it owns/consumes
Owns tool selection and stage mapping for FASTQ; consumes stage contracts and core IDs.

## Effects & determinism guarantees
Pure planning: stable ordering and hashes are enforced by snapshot tests. See `docs/TOOL_SELECTION.md`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/determinism.rs`, `tests/graph_snapshots.rs`, `tests/explainability.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/TOOL_SELECTION.md`, `docs/EXPLAIN_OUTPUT.md`, and `docs/STAGE_MAPPING.md`.

## Artifacts / Contracts
Produces plan JSON and explain payloads; snapshots live in `tests/snapshots/`.

## Failure modes
Unstable ordering or missing explain fields fail determinism and explainability tests.

## Stability
Planner outputs are snapshot-tested and versioned; see `docs/CHANGE_RULES.md`.
