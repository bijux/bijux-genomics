# bijux-domain-fastq

## What this crate does
Defines FASTQ domain truth: IDs, params, metric semantics, and invariants.

## What it must not do (boundaries)
Must not select tools or execute commands. It is a truth/semantics layer only.

## Public API / entrypoints
Domain types are documented in `docs/DOMAIN_MODEL.md`, `docs/METRICS.md`, and `docs/PARAMS.md`.

## Key contracts it owns/consumes
Owns FASTQ domain semantics; consumes core IDs and metrics registry.

## Effects & determinism guarantees
Pure data and validation; canonical serialization is enforced by `tests/params_canonical.rs`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/retention_semantics.rs`, `tests/params_canonical.rs`, `tests/determinism.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/DOMAIN_MODEL.md`, `docs/METRICS.md`, and `docs/STAGES.md`.

## Artifacts / Contracts
Defines domain JSON shapes; fixtures live in `tests/fixtures/`.

## Failure modes
Semantics violations fail invariants and retention truth-table tests.

## Stability
Domain contracts are snapshot-tested; see `docs/CHANGE_RULES.md`.
