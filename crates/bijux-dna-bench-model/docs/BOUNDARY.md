# bijux-dna-bench-model Boundary Contract

Owner: Benchmark model
Scope: Pure benchmark model types, schemas, scoring metadata, and manifests
Allowed inputs: typed benchmark payloads, profile manifests, deterministic fixtures
Forbidden dependencies: runner backends, CLI adapters, product execution crates
Forbidden effects: filesystem writes, process spawning, network access, runtime mutation
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --no-default-features`

## Why this crate exists
Defines benchmark data contracts without owning benchmark execution or reporting side effects.

## Allowed dependencies
- Core, stage-contract, analyze models, policy, and testkit support needed for schema validation.

## Allowed effects
- Pure deterministic model construction and fixture-backed schema tests.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
