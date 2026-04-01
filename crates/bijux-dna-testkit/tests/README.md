# bijux-dna-testkit Test Taxonomy

Stable test entrypoints:
- `boundaries.rs` for boundary and source-tree checks.
- `contracts.rs` for reserved contract coverage.
- `determinism.rs` for reserved determinism coverage.
- `guardrails.rs` for crate-local guardrail smoke coverage.
- `schemas.rs` for public API and snapshot normalization contracts.

Intent directories:
- `boundaries/` for dependency and layout boundaries.
- `contracts/` for contract-oriented coverage.
- `determinism/` for reproducibility coverage.
- `schemas/` for public API and normalization checks.
- `snapshots/` for locked schema snapshots.
