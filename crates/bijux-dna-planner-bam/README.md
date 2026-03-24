# bijux-dna-planner-bam

## What this crate does
BAM planner: selects tools across pre/core/downstream phases and emits plan + explain payloads.

## Phase ownership
- **Pre**: alignment validation and initial QC (planner owns selection; stages define contracts).
- **Core**: core BAM processing and QC (planner owns selection; stages define contracts).
- **Downstream**: aDNA and population analyses (planner owns selection; stages define contracts).

Planner owns selection + graph construction. Domain/stages own stage ids, artifact contracts, and metrics.

## Explainability guarantee
Explain output includes defaults diff, reasons for tool selection, and contract hashes.
See `crates/bijux-dna-planner-bam/docs/EXPLAIN_OUTPUT.md`.

## What it must not do (boundaries)
No parsing or execution.

## Role in the stack
Upstream: pipelines. Downstream: engine.

## Public API / entrypoints
See `crates/bijux-dna-planner-bam/docs/INDEX.md`, `crates/bijux-dna-planner-bam/docs/PLANNER_MODEL.md`, `crates/bijux-dna-planner-bam/docs/TOOL_SELECTION.md`, `crates/bijux-dna-planner-bam/docs/TOOL_ROSTER.md`,
`crates/bijux-dna-planner-bam/docs/EXPLAIN_OUTPUT.md`, `crates/bijux-dna-planner-bam/docs/STAGE_MAPPING.md`, `crates/bijux-dna-planner-bam/docs/ADD_TOOL.md`, `crates/bijux-dna-planner-bam/docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Plan JSON and explain payload snapshots.

## Artifacts / Contracts
See `crates/bijux-dna-planner-bam/docs/PLANNER_MODEL.md`, `crates/bijux-dna-planner-bam/docs/EXPLAIN_OUTPUT.md`, and snapshots under `tests/snapshots/`.

## Effects & determinism guarantees
Pure planning; deterministic ordering/hashes. See `crates/bijux-dna-planner-bam/docs/DETERMINISM.md` and the golden tests below.

## How to run its tests
See `crates/bijux-dna-planner-bam/docs/TESTS.md`. Golden tests: `tests/determinism.rs`, `tests/contracts/graph.rs`,
`tests/explain.rs`, `tests/plan.rs`.

## Start here in code
`src/lib.rs` → `src/selection/tool_selection.rs` → `src/tool_adapters/bam.rs`

## Where the docs live
Start at `crates/bijux-dna-planner-bam/docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-planner-bam/docs/CHANGE_RULES.md`.
