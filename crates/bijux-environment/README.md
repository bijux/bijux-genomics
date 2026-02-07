# bijux-environment

## What this crate does
Resolves tool image specs and environment references deterministically, producing pinned digests and schemas for execution.

## What it must not do (boundaries)
Must not execute tools or spawn containers. It is configuration and resolution only.

## Public API / entrypoints
Environment resolution types and helpers documented in `docs/ENV_REFERENCE.md` and `docs/SCHEMAS.md`.

## Key contracts it owns/consumes
Owns environment spec schemas; consumes core IDs for tooling references.

## Effects & determinism guarantees
No execution effects; resolution is deterministic and fixture-backed. See `docs/BOUNDARY.md` and `tests/reference_matrix.rs`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/reference_matrix.rs`, `tests/schema_snapshots.rs`, `tests/guardrails_runtime.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/ENV_REFERENCE.md`, `docs/ENV_MATRIX.md`, and `docs/SCHEMAS.md`.

## Artifacts / Contracts
Produces resolved image specs and digests; no runtime artifacts.

## Failure modes
Invalid specs or mismatched digests are reported by resolution tests and schema validation.

## Stability
Schemas are snapshot-tested and versioned; see `docs/CHANGE_RULES.md`.
