# Tests

## What
Maps the stable test entrypoints and intent directories for the testkit crate.

## Why
The crate is small, so the test tree should be explicit and durable.

## Entry points
- `tests/boundaries.rs` — boundary and source-tree guardrails.
- `tests/contracts.rs` — reserved contract-suite entrypoint for future test-support contracts.
- `tests/determinism.rs` — reserved determinism-suite entrypoint for future reproducibility checks.
- `tests/guardrails.rs` — crate-local guardrail smoke test.
- `tests/schemas.rs` — public API, docs, and snapshot normalization checks.

## Intent directories
- `tests/boundaries/` — dependency and layout boundaries.
- `tests/contracts/` — contract-oriented tests and fixtures.
- `tests/determinism/` — determinism-focused tests and notes.
- `tests/schemas/` — public API and normalization contracts.
- `tests/snapshots/` — locked snapshots for public surface checks.

## Source-tree contract
- `tests/boundaries/architecture_tree.rs` locks the crate tree to the documented namespace layout.
