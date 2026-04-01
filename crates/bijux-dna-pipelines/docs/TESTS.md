# Tests

## What
Maps the stable test entrypoints and intent directories for bijux-dna-pipelines.

## Why
Pipeline profiles are consumed across the stack, so registry drift and layout drift need explicit locks.

## Entry points
- `tests/boundaries.rs` — boundary and source-tree contract coverage.
- `tests/contracts.rs` — defaults, profiles, and registry contract coverage.
- `tests/guardrails.rs` — crate-local guardrail smoke coverage.
- `tests/invariant_fast.rs` — fast invariant checks.

## Intent directories
- `tests/boundaries/` — architecture and guardrail coverage.
- `tests/contracts/` — defaults, profile, and registry contracts.
- `tests/determinism/` — reserved determinism notes and future reproducibility coverage.
- `tests/schemas/` — reserved docs and public-surface lock coverage.

## Source-tree contract
- `tests/boundaries/architecture_tree.rs` locks the documented `pipelines` namespace layout.
