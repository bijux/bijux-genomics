# bijux-dna-testkit

## What this crate does
Provides deterministic test support for fixtures, snapshots, temporary paths, and workspace-aware test helpers.
The tree is partitioned by support concern rather than by generic helper buckets.

## What it must not do (boundaries)
No domain logic or production dependencies.
This crate must remain a test-only support crate with explicit namespaces and no product runtime behavior.

## Role in the stack
Upstream: none. Downstream: tests across workspace.

## Public API / entrypoints
- Stable root re-exports cover `FixedClock`, `fixed_rng`, fixture readers, snapshot normalization, temporary path helpers, and workspace support.
- Public modules are `determinism`, `fixtures`, `public_api`, `snapshots`, `temp`, and `workspace_support`.
- `public_api/` mirrors the curated root surface so internal layout can evolve behind one stable namespace.

## Key contracts it owns/consumes
Owns the deterministic snapshot normalization contract, fixture-reading helpers, and temporary-path support used across the workspace test suites.

## Artifacts / Contracts
See `crates/bijux-dna-testkit/docs/FIXTURE_STANDARDS.md`, `crates/bijux-dna-testkit/docs/SNAPSHOT_POLICY.md`, `crates/bijux-dna-testkit/docs/PUBLIC_API.md`, and `crates/bijux-dna-testkit/docs/ARCHITECTURE.md`.

## Effects & determinism guarantees
Test-only filesystem helpers and deterministic normalization. Snapshot text, snapshot JSON, fixture reading, and temp-path helpers are owned in separate source namespaces.

## How to run its tests
See `crates/bijux-dna-testkit/docs/TESTS.md`. Stable entrypoints are `tests/boundaries.rs`, `tests/contracts.rs`, `tests/determinism.rs`, `tests/guardrails.rs`, and `tests/schemas.rs`.

## Where the docs live
Start at `crates/bijux-dna-testkit/docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/lib.rs` → `src/public_api/` for surface contracts, then `src/snapshots/`, `src/determinism/`, `src/fixtures/`, `src/temp/`, and `src/workspace_support/`.

## Failure modes
Primary failures surface as public-surface drift, snapshot normalization regressions, or test-support layout violations.

## Stability
Contract and behavior changes follow `crates/bijux-dna-testkit/docs/CHANGE_RULES.md`.
