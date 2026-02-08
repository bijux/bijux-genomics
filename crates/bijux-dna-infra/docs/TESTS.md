# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- Each test file should be referenced here.

## Examples
- `tests/determinism/hash.rs` → stable hashing contract.

## Failure modes
- Missing test documentation causes drift and confusion.

## determinism
- `tests/determinism/hash.rs` — hashing determinism for file inputs.

## guardrails
- `tests/guardrails/canonical_owner.rs` — PATHS doc must point to bijux-dna-core.
- `tests/guardrails/no_generic_helpers.rs` — no generic helper-y API creep.
- `tests/guardrails/policies.rs` — shared policy guardrails.
- `tests/public_surface.rs` — public API surface snapshot.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
