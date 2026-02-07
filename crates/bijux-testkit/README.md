# bijux-testkit

## What this crate does
Provides deterministic helpers for fixtures and snapshots used across workspace tests.

## What it must not do (boundaries)
Must not contain domain logic or production dependencies. It is test-only and lightweight.

## Public API / entrypoints
Helper patterns are documented in `docs/USAGE.md` and `docs/FIXTURE_STANDARDS.md`.

## Key contracts it owns/consumes
Consumes core canonicalization and provides helper wrappers; no standalone contracts.

## Effects & determinism guarantees
Helpers must be deterministic and stable; see `docs/SNAPSHOT_POLICY.md`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/docs_lightweight.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/FIXTURE_STANDARDS.md` and `docs/SNAPSHOT_POLICY.md`.

## Artifacts / Contracts
No runtime artifacts; used by test fixtures only.

## Failure modes
Misuse is caught by lightweight guardrail tests.

## Stability
Helpers are stable; changes require updates to docs and tests per `docs/CHANGE_RULES.md`.
