# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- Each test file should be referenced here.

## Suite map
- `tests/observer/*` → observer parser fixtures under `tests/fixtures/*`.
- `tests/contracts/*` → stage specs, registry, symmetry, and contract snapshots.
- `tests/purity/*` → architecture and purity checks.

## Examples
- `tests/observer/observer_parsers.rs` → observer fixture parsing.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
