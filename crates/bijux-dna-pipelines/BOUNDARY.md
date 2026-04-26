# bijux-dna-pipelines Boundary Contract

Owner: Pipelines
Scope: Default pipeline composition, profile manifests, and defaults ledger authority
Allowed inputs: domain contracts, typed pipeline IDs, deterministic profile fixtures
Forbidden dependencies: runner backends, CLI adapters, direct tool execution
Forbidden effects: product execution, process spawning, network access, undeclared file writes
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-pipelines --no-default-features`

## Why this crate exists
Owns deterministic pipeline profile composition and profile manifest/defaults-ledger contracts.

## Allowed dependencies
- Domain, core, runtime model, and policy/testkit crates needed to assemble profile contracts.
- No backend execution or command-surface ownership.

## Allowed effects
- Pure deterministic profile construction and fixture-backed tests.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
