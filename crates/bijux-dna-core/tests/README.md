# bijux-dna-core Test Taxonomy

Stable test entrypoints:
- `boundaries.rs` for layering, guardrails, and source-tree contracts.
- `contracts.rs` for execution, identity, and public contract behavior.
- `schemas.rs` for public API and docs locks.
- `semantics.rs` for identifier, metric, and input-assessment semantics.
- `guardrails.rs` for crate-local guardrail smoke coverage.

Intent directories:
- `boundaries/` for layering and architecture contracts.
- `contracts/` for behavior contracts.
- `determinism/` for reproducibility notes and future coverage.
- `fixtures/` for stable test inputs shared across entrypoints.
- `schemas/` for public-surface and docs locks.
- `semantics/` for semantic behavior checks, including canonical identifier family coverage.
