# bijux-stages-bam

## What this crate does
BAM stage specs + observers only, organized by pre/core/downstream phases.

## What it must not do (boundaries)
No command assembly or tool selection.

## Role in the stack
Upstream: domain contracts. Downstream: planners/analyze.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/PHASES.md`, `docs/STAGE_LIST.md`, `docs/STAGE_CONTRACTS.md`, `docs/OBSERVERS.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Stage report/metrics shape snapshots.

## Effects & determinism guarantees
Pure parsing; deterministic snapshots. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/contract_snapshots.rs`, `tests/observer_determinism.rs`, `tests/metrics_completeness.rs`, `tests/structure_contract.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
