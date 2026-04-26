# bijux-dna-testkit Boundary Contract

Owner: Testkit
Scope: Test fixtures, deterministic sanitizers, snapshot helpers, and workspace path helpers
Allowed inputs: test fixtures, snapshot text/json, repository paths
Forbidden dependencies: production runtime/runner as required dependencies, CLI adapters
Forbidden effects: production execution, network access, source mutation
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --no-default-features`

## Why this crate exists
Provides reusable deterministic testing helpers without becoming a product dependency owner.

## Allowed dependencies
- Policy-neutral dependencies and test-only support.
- No production execution ownership.

## Allowed effects
- Test helper reads and deterministic sanitization.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
