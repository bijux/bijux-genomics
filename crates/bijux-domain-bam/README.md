# bijux-domain-bam

## What this crate does
Defines BAM domain truth: phase model (pre/core/downstream), params, metric semantics, and invariants.

## What it must not do (boundaries)
Must not select tools or execute commands. It is a truth/semantics layer only.

## Public API / entrypoints
Domain types are documented in `docs/DOMAIN_MODEL.md`, `docs/PHASES.md`, and `docs/METRICS.md`.

## Key contracts it owns/consumes
Owns BAM domain semantics; consumes core IDs and metrics registry.

## Effects & determinism guarantees
Pure data and validation; canonical serialization is enforced by `tests/canonical_serialization.rs`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/phase_semantics.rs`, `tests/metrics_contract.rs`, `tests/canonical_serialization.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/PHASES.md`, `docs/METRICS.md`, and `docs/PARAMS.md`.

## Artifacts / Contracts
Defines domain JSON shapes; fixtures live in `tests/fixtures/`.

## Failure modes
Semantics violations fail phase and completeness tests.

## Stability
Domain contracts are snapshot-tested; see `docs/CHANGE_RULES.md`.
