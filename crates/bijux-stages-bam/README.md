# bijux-stages-bam

## What this crate does
Defines BAM stage specs and observers only, organized by pre/core/downstream phases.

## What it must not do (boundaries)
Must not assemble commands or select tools. Execution belongs in planners/runner.

## Public API / entrypoints
Stage specs and observers documented in `docs/PHASES.md`, `docs/STAGE_LIST.md`, and `docs/OBSERVERS.md`.

## Key contracts it owns/consumes
Owns BAM stage contracts; consumes core IDs and domain semantics.

## Effects & determinism guarantees
Parsing is deterministic and fixture-backed. See `tests/observer_determinism.rs`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/contract_snapshots.rs`, `tests/observer_determinism.rs`, `tests/metrics_completeness.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/PHASES.md`, `docs/STAGE_CONTRACTS.md`, and `docs/REFERENCES.md`.

## Artifacts / Contracts
Produces stage_report/metrics shapes via parsers; snapshots live in `tests/fixtures/observer_snapshots/`.

## Failure modes
Parser regressions or contract drift fail snapshot tests.

## Stability
Stage contracts and observer outputs are snapshot-tested; see `docs/CHANGE_RULES.md`.
