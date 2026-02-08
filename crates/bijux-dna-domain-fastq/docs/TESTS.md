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
- `tests/contracts/*` → stage contracts, SSOT ids, and public surface.
- `tests/invariants/*` → invariant specs and invariant enforcement.
- `tests/semantics/*` → retention semantics, params canonicalization, observability.
- `tests/purity/*` → architecture, determinism, and guardrails.

## Examples
- `tests/contracts/stage_contract_snapshots.rs` → stage contract snapshots.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
