# bijux-domain-bam

## What this crate does
BAM domain truth: phase model, params, metric semantics, invariants.

## What it must not do (boundaries)
No selection or execution.

## Role in the stack
Upstream: core IDs. Downstream: planners/stages/analyze.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/PHASES.md`, `docs/METRICS.md`, `docs/PARAMS.md`, `docs/DOMAIN_MODEL.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Domain JSON shapes and fixtures.

## Effects & determinism guarantees
Pure data/validation. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/phase_semantics.rs`, `tests/metrics_contract.rs`, `tests/canonical_serialization.rs`, `tests/reference_suite.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
