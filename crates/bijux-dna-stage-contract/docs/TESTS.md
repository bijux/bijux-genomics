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
- `tests/schema/*` → public types and schema snapshots from `tests/fixtures/public_types/*`.
- `tests/versioning/*` → versioning and SSOT checks.
- `tests/guardrails/*` → no-execution scans and tree contracts.

## No execution scan
The no-execution scan forbids process spawning and runtime effects in this crate.

## Examples
- `tests/schema/public_type_snapshots.rs` → public surface snapshots.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
