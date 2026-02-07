# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- Each test file should be referenced here.
- Fixtures under `tests/fixtures/*` back the contract snapshots and golden comparisons.

## Suite map
- `tests/contracts/*` → boundary, API surface, and schema contract checks.
- `tests/determinism/*` → deterministic ordering and snapshot stability.
- `tests/gate/*` → policy and gating invariants.

## Examples
- `tests/contracts/architecture.rs` → dependency boundary assertions.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
