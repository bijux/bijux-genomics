# bijux-dna-planner-fastq

## What this crate does
FASTQ planner: selects tools and generates graphs + explain payloads.
Stage grouping is explicit: `pre` (validation + stats), `qc` (quality reports),
and `transform` (trim/merge/filter/correct/umi/screen).

## Selection + explainability
Selection inputs:
- pipeline id + profile overrides
- allow/deny tool lists (CLI/API)
- stage availability from domain contracts
- explicit tool version pins (if provided)

Explain output guarantees:
- lists chosen tools in stable order
- includes defaults diff (profile vs pipeline)
- includes reasons for each selection
- includes contract hashes for plan stability

## What it must not do (boundaries)
No parsing or execution.

## Role in the stack
Upstream: pipelines. Downstream: engine.

## Public API / entrypoints
See `crates/bijux-dna-planner-fastq/docs/INDEX.md`, `crates/bijux-dna-planner-fastq/docs/PLANNER_MODEL.md`, `crates/bijux-dna-planner-fastq/docs/TOOL_SELECTION.md`, `crates/bijux-dna-planner-fastq/docs/EXPLAIN_OUTPUT.md`,
`crates/bijux-dna-planner-fastq/docs/STAGE_MAPPING.md`, `crates/bijux-dna-planner-fastq/docs/ADD_TOOL.md`, `crates/bijux-dna-planner-fastq/docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Plan JSON and explain payload snapshots.

## Artifacts / Contracts
See `crates/bijux-dna-planner-fastq/docs/PLANNER_MODEL.md`, `crates/bijux-dna-planner-fastq/docs/EXPLAIN_OUTPUT.md`, and snapshots under `tests/snapshots/`.

## Effects & determinism guarantees
Pure planning; deterministic ordering/hashes. See `crates/bijux-dna-planner-fastq/docs/DETERMINISM.md` and the golden tests below.

## How to run its tests
See `crates/bijux-dna-planner-fastq/docs/TESTS.md`. Golden tests: `tests/determinism.rs`, `tests/contracts/graph.rs`,
`tests/explain.rs`, `tests/plan.rs`.

## Start here in code
`src/lib.rs` → `src/selection/tool_selection.rs` → `src/tool_adapters/fastq.rs`

## Where the docs live
Start at `crates/bijux-dna-planner-fastq/docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-planner-fastq/docs/CHANGE_RULES.md`.
