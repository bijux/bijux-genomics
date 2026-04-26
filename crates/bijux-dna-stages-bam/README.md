# bijux-dna-stages-bam

## What this crate does
BAM stage specs + observers only, organized by pre/core/downstream phases.

## What it must not do (boundaries)
No command assembly or tool selection.

## Role in the stack
Upstream: domain contracts. Downstream: planners/analyze.

## Public API / entrypoints
See `crates/bijux-dna-stages-bam/docs/INDEX.md`, `crates/bijux-dna-stages-bam/docs/PHASES.md`, `crates/bijux-dna-stages-bam/docs/STAGE_LIST.md`, `crates/bijux-dna-stages-bam/docs/STAGE_CONTRACTS.md`, `crates/bijux-dna-stages-bam/docs/OBSERVERS.md`, `crates/bijux-dna-stages-bam/docs/CHANGE_RULES.md`.

## Phases and observer responsibilities
- **Pre**: validation + alignment QC outputs.
- **Core**: core BAM processing metrics (markdup, coverage, depth).
- **Downstream**: aDNA and population analyses (damage, contamination, sex).

Observers parse only documented tool outputs, ignore unknown fields, and require contract fields.

## Key contracts it owns/consumes
Stage report/metrics shape snapshots.

## Artifacts / Contracts
See `crates/bijux-dna-stages-bam/docs/STAGE_CONTRACTS.md`, `crates/bijux-dna-stages-bam/docs/OBSERVERS.md`, and contract snapshots under `tests/contracts/`.

## Effects & determinism guarantees
Pure parsing; deterministic snapshots. See `crates/bijux-dna-stages-bam/docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `crates/bijux-dna-stages-bam/docs/TESTS.md`. Golden tests: `tests/contracts/contract_snapshots.rs`, `tests/contracts/observer/observer_determinism.rs`, `tests/semantics/metrics/metrics_completeness.rs`, `tests/contracts/structure_contract.rs`.

## Where the docs live
Start at `crates/bijux-dna-stages-bam/docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/stage_specs.rs` → `src/observer.rs` → `src/plugin/`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-stages-bam/docs/CHANGE_RULES.md`.
