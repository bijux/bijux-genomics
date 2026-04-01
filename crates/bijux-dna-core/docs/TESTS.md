# Tests

## What
Maps the stable test entrypoints and intent directories for bijux-dna-core.

## Why
Core is a dependency anchor, so source-tree drift and public-surface drift need explicit guardrails.

## Entry points
- `tests/boundaries.rs` — boundary, layering, guardrail, and source-tree contract coverage.
- `tests/contracts.rs` — contract behavior, identity, execution, and surface contracts.
- `tests/schemas.rs` — docs and public-surface locks.
- `tests/semantics.rs` — identifier, metrics, and input-assessment semantics.
- `tests/guardrails.rs` — crate-local guardrail smoke coverage.

## Intent directories
- `tests/boundaries/` — dependency boundaries and layout contracts.
- `tests/contracts/` — execution, identity, and surface behavior contracts.
- `tests/determinism/` — reserved determinism notes and future reproducibility coverage.
- `tests/schemas/` — public API and docs locks.
- `tests/semantics/` — semantic behavior checks for IDs, metrics, and input assessment.

## Source-tree contract
- `tests/boundaries/architecture_tree.rs` locks the documented `core` namespace layout.
