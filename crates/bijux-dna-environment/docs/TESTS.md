# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Schema suite (`tests/schema/*`)
- `tests/schema/schema_snapshots.rs` → schema snapshot stability.

## Matrix suite (`tests/matrix/*`)
- `tests/matrix/reference_matrix.rs` → resolution matrix coverage.
- `tests/matrix/docs_reference_matrix.rs` → docs reference matrix coverage.

## Guardrails suite (`tests/guardrails/*`)
- `tests/guardrails/guardrails.rs` → boundary checks.
- `tests/guardrails/guardrails_runtime.rs` → runtime guardrails.
- `tests/guardrails/no_runner_usage.rs` → no runner dependency.

## Fixtures mapping
Schema fixtures in `tests/fixtures/env_schema/*` are validated by:
- `tests/schema/schema_snapshots.rs`
- `tests/matrix/reference_matrix.rs`

## Failure modes
- Missing test documentation causes drift and confusion.
