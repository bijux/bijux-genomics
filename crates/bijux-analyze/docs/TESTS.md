# Tests

## What
Maps tests in this crate to their purpose and failure meaning.

## Why
Tests should explain the contract they enforce.

## Non-goals
- Full test implementation detail.

## Contracts
- Each test file should be referenced here.
- `tests/fixtures/golden_spine/*` is the canonical reference dataset for report contract coverage.

## Suite map
- `tests/contracts/*` → boundary, schema, and registry invariants (facts/metrics/contracts/guardrails).
- `tests/decision/*` → ranking, scoring, and decision trace invariants.
- `tests/report/*` → report artifacts, determinism, privacy, and performance budgets.
- `tests/sqlite/*` → SQLite storage compatibility and migrations.
- `tests/pipeline/*` → end-to-end load → decide → report wiring and stage boundaries.

## Examples
- `tests/contracts/architecture.rs` → dependency boundary assertions.
- `tests/report/report_contract.rs` → report artifact schema stability.

## Failure modes
- Missing test documentation causes drift and confusion.

## Testkit patterns
See `crates/bijux-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
