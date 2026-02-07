# bijux-domain-fastq

## What this crate does
FASTQ domain truth: IDs, params, metric semantics, invariants.

## What it must not do (boundaries)
No selection or execution.

## Role in the stack
Upstream: core IDs. Downstream: planners/stages/analyze.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/DOMAIN_MODEL.md`, `docs/METRICS.md`, `docs/PARAMS.md`, `docs/STAGES.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Domain JSON shapes and fixtures.

## Effects & determinism guarantees
Pure data/validation. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/retention_semantics.rs`, `tests/params_canonical.rs`, `tests/retention_truth_table.rs`, `tests/determinism.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
