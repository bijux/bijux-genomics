# bijux-planner-bam

## What this crate does
BAM planner: selects tools across pre/core/downstream phases and emits plan + explain payloads.

## Phase ownership
- **Pre**: alignment validation and initial QC (planner owns selection; stages define contracts).
- **Core**: core BAM processing and QC (planner owns selection; stages define contracts).
- **Downstream**: aDNA and population analyses (planner owns selection; stages define contracts).

Planner owns selection + graph construction. Domain/stages own stage ids, artifact contracts, and metrics.

## Explainability guarantee
Explain output includes defaults diff, reasons for tool selection, and contract hashes.
See `docs/EXPLAIN_OUTPUT.md`.

## What it must not do (boundaries)
No parsing or execution.

## Role in the stack
Upstream: pipelines. Downstream: engine.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/PLANNER_MODEL.md`, `docs/TOOL_SELECTION.md`, `docs/TOOL_ROSTER.md`,
`docs/EXPLAIN_OUTPUT.md`, `docs/STAGE_MAPPING.md`, `docs/ADD_TOOL.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Plan JSON and explain payload snapshots.

## Artifacts / Contracts
See `docs/PLANNER_MODEL.md`, `docs/EXPLAIN_OUTPUT.md`, and snapshots under `tests/snapshots/`.

## Effects & determinism guarantees
Pure planning; deterministic ordering/hashes. See `docs/DETERMINISM.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/determinism.rs`, `tests/contracts/graph.rs`,
`tests/explain.rs`, `tests/plan.rs`.

## Start here in code
`src/lib.rs` → `src/selection/tool_selection.rs` → `src/tool_adapters/bam.rs`

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
