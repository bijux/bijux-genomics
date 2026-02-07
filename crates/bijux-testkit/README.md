# bijux-testkit

## What this crate does
Deterministic test helpers for fixtures/snapshots.

## What it must not do (boundaries)
No domain logic or production dependencies.

## Role in the stack
Upstream: none. Downstream: tests across workspace.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/FIXTURE_STANDARDS.md`, `docs/SNAPSHOT_POLICY.md`, `docs/USAGE.md`, `docs/ARCHITECTURE.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Test-only helpers.

## Effects & determinism guarantees
Test-only utilities; deterministic output. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/docs_lightweight.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
