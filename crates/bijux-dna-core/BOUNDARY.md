# bijux-dna-core Boundary Contract

Owner: Core
Scope: Pure shared IDs, typed contracts, manifests, hashing, and stable model primitives
Allowed inputs: typed values, serialized contract payloads, deterministic test fixtures
Forbidden dependencies: planner, runner, engine, API, CLI, environment, and analyzer crates
Forbidden effects: filesystem writes, process spawning, network access, runtime mutation
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --no-default-features`

## Why this crate exists
Defines the foundational data contracts shared by the workspace without owning orchestration or
runtime behavior.

## Allowed dependencies
- Policy-neutral infra and testkit support where required by tests.
- No reverse-layer coupling into product execution or command surfaces.

## Allowed effects
- Pure deterministic computation only.
- Tests may read fixtures through testkit helpers.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
