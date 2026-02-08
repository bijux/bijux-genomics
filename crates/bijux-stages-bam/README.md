# bijux-stages-bam

## What this crate does
BAM stage specs + observers only, organized by pre/core/downstream phases.

## What it must not do (boundaries)
No command assembly or tool selection.

## Role in the stack
Upstream: domain contracts. Downstream: planners/analyze.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/PHASES.md`, `docs/STAGE_LIST.md`, `docs/STAGE_CONTRACTS.md`, `docs/OBSERVERS.md`, `docs/CHANGE_RULES.md`.

## Phases and observer responsibilities
- **Pre**: validation + alignment QC outputs.
- **Core**: core BAM processing metrics (markdup, coverage, depth).
- **Downstream**: aDNA and population analyses (damage, contamination, sex).

Observers parse only documented tool outputs, ignore unknown fields, and require contract fields.

## Key contracts it owns/consumes
Stage report/metrics shape snapshots.

## Effects & determinism guarantees
Pure parsing; deterministic snapshots. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/contracts/contract_snapshots.rs`, `tests/observer/observer_determinism.rs`, `tests/metrics/metrics_completeness.rs`, `tests/contracts/structure_contract.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/stage_specs.rs` → `src/observer.rs` → `src/plugin.rs`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
