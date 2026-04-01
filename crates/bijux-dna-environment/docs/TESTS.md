# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts suite (`tests/contracts/*`)
- `tests/contracts/build_dockerfile.rs` → Dockerfile version parsing coverage.
- `tests/contracts/run_shell_capture.rs` → shell capture behavior.
- `tests/contracts/resolve_runtime.rs` → resolver, runtime, cache, and catalog contracts.
- `tests/contracts/matrix/reference_matrix.rs` → resolution matrix coverage.
- `tests/contracts/matrix/docs_reference_matrix.rs` → reference-doc fixture coverage.

## Boundaries suite (`tests/boundaries/*`)
- `tests/boundaries/architecture.rs` → source tree contract.
- `tests/boundaries/guardrails/guardrails.rs` → boundary checks.
- `tests/boundaries/guardrails/guardrails_runtime.rs` → runtime guardrails.
- `tests/boundaries/guardrails/no_runner_usage.rs` → no runner dependency.

## Determinism suite (`tests/determinism/*`)
- `tests/determinism/fixture_stability.rs` → stable fixture output checks.

## Schemas suite (`tests/schemas/*`)
- `tests/schemas/schema/schema_snapshots.rs` → schema snapshot stability.

## Fixtures mapping
Schema fixtures in `tests/fixtures/env_schema/*` are validated by:
- `tests/schemas/schema/schema_snapshots.rs`
- `tests/contracts/matrix/reference_matrix.rs`

## Failure modes
- Missing test documentation causes drift and confusion.
