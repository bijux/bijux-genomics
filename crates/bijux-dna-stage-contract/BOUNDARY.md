# bijux-dna-stage-contract Boundary Contract

Owner: Stage contract
Scope: Shared stage schema, invocation, artifact, and metric contract types
Allowed inputs: typed core IDs, serialized stage contract payloads, deterministic schema fixtures
Forbidden dependencies: runner backends, CLI adapters, planner selection logic, API orchestration
Forbidden effects: filesystem writes, process spawning, network access, runtime mutation
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --no-default-features`

## Why this crate exists
Defines shared stage contract data structures used by planners, stages, and policy checks.

## Allowed dependencies
- Core and policy/testkit support needed to express and validate schemas.
- No execution or orchestration ownership.

## Allowed effects
- Pure deterministic schema construction and validation.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
