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

## Commands

Use artifact-rooted target and temp directories:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-infra --no-default-features
```

`temp_dir` honors `TEST_TMP_DIR`, so local test runs keep temporary directories under the repository
artifact root instead of the OS temp root.

## Failure modes
- Missing test documentation causes drift and confusion.

## Determinism
- `tests/determinism/hash.rs` — hashing determinism for file inputs.
- `tests/determinism/retry.rs` — retry backoff sequence remains stable.

## Boundaries
- `tests/boundaries/architecture.rs` — crate tree, docs allowance, source ownership, and test taxonomy.
- `tests/boundaries/guardrails/canonical_owner.rs` — PATHS doc must point to bijux-dna-core.
- `tests/boundaries/guardrails/dependencies.rs` — runtime dependencies must match the documented
  low-level dependency boundary.
- `tests/boundaries/guardrails/no_generic_helpers.rs` — no generic helper-y API creep.
- `tests/boundaries/guardrails/policies.rs` — shared policy guardrails.
- `tests/boundaries/guardrails/docs_layout.rs` — docs must stay aligned with the current crate tree.

## Schemas
- `tests/schemas/public_surface.rs` — public API surface snapshot.

## Testkit patterns
See `crates/bijux-dna-testkit/docs/SNAPSHOT_POLICY.md` for shared fixture and
snapshot stability rules.
