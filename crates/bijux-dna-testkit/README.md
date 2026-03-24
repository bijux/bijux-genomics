# bijux-dna-testkit

## What this crate does
Deterministic test helpers for fixtures and snapshots.
Standardizes:
- fixture layout conventions
- canonical JSON ordering
- deterministic filesystem helpers

## What it must not do (boundaries)
No domain logic or production dependencies.
This crate must remain a tiny, test-only helper.
Hard rule: testkit must not depend on engine or runner crates.

## Role in the stack
Upstream: none. Downstream: tests across workspace.

## Public API / entrypoints
See `crates/bijux-dna-testkit/docs/INDEX.md`, `crates/bijux-dna-testkit/docs/PUBLIC_API.md`, `crates/bijux-dna-testkit/docs/FIXTURE_STANDARDS.md`,
`crates/bijux-dna-testkit/docs/ADD_FIXTURE.md`, `crates/bijux-dna-testkit/docs/SNAPSHOT_POLICY.md`, `crates/bijux-dna-testkit/docs/USAGE.md`,
`crates/bijux-dna-testkit/docs/ARCHITECTURE.md`, `crates/bijux-dna-testkit/docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Test-only helpers.

## Artifacts / Contracts
See `crates/bijux-dna-testkit/docs/FIXTURE_STANDARDS.md`, `crates/bijux-dna-testkit/docs/SNAPSHOT_POLICY.md`, and `crates/bijux-dna-testkit/docs/PUBLIC_API.md`.

## Effects & determinism guarantees
Test-only utilities; deterministic output. See `crates/bijux-dna-testkit/docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `crates/bijux-dna-testkit/docs/TESTS.md`. Tests: `tests/docs_lightweight.rs`, `tests/public_api_surface.rs`,
`tests/public_api_snapshot.rs`, `tests/dev_dep_boundary.rs`.

## Where the docs live
Start at `crates/bijux-dna-testkit/docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/lib.rs` → `src/snapshots` (canonical JSON ordering)

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-testkit/docs/CHANGE_RULES.md`.
