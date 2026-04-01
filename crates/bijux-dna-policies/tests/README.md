# Policy Tests

Stable test entrypoints:
- `boundaries.rs` for boundary, surface, and workspace layout policies.
- `contracts.rs` for contract, tooling, and governance policies.
- `determinism.rs` for stable-order and fixture reproducibility policies.
- `guardrails.rs` for crate-local guardrail wiring.

Intent directories:
- `boundaries/` for dependency boundaries, documentation spine rules, and source layout policies.
- `contracts/` for contract fixtures, snapshots, tooling governance, and workspace-wide invariants.
- `determinism/` for reproducibility checks.
- `support/`, `fixtures/`, `snapshots/`, and `schemas/` for shared artifacts and helpers.
