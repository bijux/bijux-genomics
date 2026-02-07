# bijux-pipelines

## What this crate does
Defines scientific pipeline presets and profiles, including defaults ledgers and profile invariants.

## What it must not do (boundaries)
Must not execute, select tools, or parse outputs. It defines declarative profiles only.

## Public API / entrypoints
Pipeline registry and profiles are documented in `docs/PIPELINES.md` and `docs/PIPELINE_MODEL.md`.

## Key contracts it owns/consumes
Owns pipeline IDs and defaults ledger semantics; consumes domain/planner contracts.

## Effects & determinism guarantees
Pure data; ordering is snapshot-tested for determinism. See `tests/pipeline_registry_snapshot.rs`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/pipeline_registry_snapshot.rs`, `tests/pipeline_completeness.rs`, `tests/override_precedence.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/PIPELINES.md`, `docs/DEFAULTS_LEDGER.md`, and `docs/PIPELINE_VERSIONING.md`.

## Artifacts / Contracts
Produces pipeline profiles and defaults ledgers (JSON snapshots in tests).

## Failure modes
Missing defaults or unstable ordering fail completeness and snapshot tests.

## Stability
Pipeline changes require versioning updates per `docs/PIPELINE_VERSIONING.md` and `docs/CHANGE_RULES.md`.
