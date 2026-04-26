# bijux-dna-infra Boundary Contract

Owner: Infra
Scope: Generic filesystem, hashing, IO, config path, and locking primitives
Allowed inputs: paths, bytes, serialized config payloads, deterministic fixtures
Forbidden dependencies: domain semantics, planner/runtime orchestration, CLI adapters
Forbidden effects: process spawning, network access, domain-specific writes
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-infra --no-default-features`

## Why this crate exists
Provides reusable policy-neutral infrastructure primitives without owning product semantics.

## Allowed dependencies
- Policy and testkit support where required by tests.
- No domain or orchestration ownership.

## Allowed effects
- Generic filesystem IO only when called by an owning layer.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
