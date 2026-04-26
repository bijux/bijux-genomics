# bijux-dna-policies Boundary Contract

Owner: Policies
Scope: Repository policy diagnostics over source, docs, configs, fixtures, and snapshots
Allowed inputs: repository files, policy fixtures, docs, generated config views
Forbidden dependencies: product execution ownership, runner backends as required runtime deps
Forbidden effects: source mutation, generated config rewrites, snapshot rewrites, network access
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --no-default-features`

## Why this crate exists
Owns deterministic repository policy checks and actionable diagnostics.

## Allowed dependencies
- Testkit and model crates required to inspect repository contracts.
- No product pipeline execution.

## Allowed effects
- Read-only repository inspection and test diagnostics.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
