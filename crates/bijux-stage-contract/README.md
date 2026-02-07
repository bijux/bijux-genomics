# bijux-stage-contract

## What this crate does
Defines minimal planning types for stage plans and plugins, separate from runtime execution contracts.

## What it must not do (boundaries)
Must not include execution details or runner/runtime concepts. It is planning-only.

## Public API / entrypoints
Public types documented in `docs/CONTRACT.md` and `docs/SCHEMAS.md`.

## Key contracts it owns/consumes
Owns stage plan and execution plan schemas; consumes core IDs.

## Effects & determinism guarantees
No effects; serialization is canonical and snapshot-tested.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/public_type_snapshots.rs`, `tests/schema_snapshots.rs`, `tests/no_execution_scan.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/CONTRACT.md`, `docs/VERSIONING.md`, and `docs/MINIMALITY.md`.

## Artifacts / Contracts
Produces plan JSON shapes; fixtures live in `tests/fixtures/public_types/`.

## Failure modes
Schema drift and versioning violations fail snapshot tests.

## Stability
Contract changes require version bumps per `docs/VERSIONING.md`.
