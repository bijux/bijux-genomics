# bijux-stages-fastq

## What this crate does
Defines FASTQ stage specs and observers (parsers) only. It is the contract+parsing layer for FASTQ stages.

## What it must not do (boundaries)
Must not assemble commands or select tools. Execution belongs in planners/runner.

## Public API / entrypoints
Stage specs and observers documented in `docs/STAGE_LIST.md`, `docs/STAGE_CONTRACTS.md`, and `docs/OBSERVERS.md`.

## Key contracts it owns/consumes
Owns FASTQ stage contracts; consumes core IDs and domain semantics.

## Effects & determinism guarantees
Parsing is deterministic and fixture-backed. See `tests/observer_determinism.rs`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/contract_snapshots.rs`, `tests/observer_determinism.rs`, `tests/symmetry.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/STAGE_LIST.md`, `docs/OBSERVERS.md`, and `docs/TOOL_ROSTER.md`.

## Artifacts / Contracts
Produces stage_report/metrics shapes via parsers; snapshots live in `tests/snapshots/`.

## Failure modes
Parser regressions or contract drift fail snapshot tests.

## Stability
Stage contracts and observer outputs are snapshot-tested; see `docs/CHANGE_RULES.md`.
