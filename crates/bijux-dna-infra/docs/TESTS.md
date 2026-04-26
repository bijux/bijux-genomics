# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- `tests/contracts/io.rs` — IO guarantees for atomic writes, bounded reads, temp dirs, and removal semantics.
- `tests/contracts/run_layout.rs` — run-layout path, lock, and publish contracts.

## Failure modes
- Missing test documentation causes drift and confusion.

## Determinism
- `tests/determinism/hash.rs` — hashing determinism for file inputs.
- `tests/determinism/retry.rs` — retry backoff sequence remains stable.

## Boundaries
- `tests/boundaries/guardrails/canonical_owner.rs` — PATHS doc must point to bijux-dna-core.
- `tests/boundaries/guardrails/dependencies.rs` — runtime dependencies must match the documented
  low-level dependency boundary.
- `tests/boundaries/guardrails/no_generic_helpers.rs` — no generic helper-y API creep.
- `tests/boundaries/guardrails/policies.rs` — shared policy guardrails.
- `tests/boundaries/guardrails/public_surface.rs` — public API surface snapshot.
- `tests/boundaries/guardrails/docs_layout.rs` — docs must stay aligned with the current crate tree.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
