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
- `tests/public_api/*` → public surface and docs linkage.
- `tests/determinism/*` → seeded randomness and deterministic outputs.
- `tests/semantics/*` → model semantics, SSOT metrics, and guardrails.

## Examples
- `tests/semantics/decision_explainability.rs` → explainability snapshots.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
