# bijux-dna-runner Boundary Contract

Owner: Runner
Scope: Backend process and container invocation boundary
Allowed inputs: explicit tool invocation requests, resolved runtime environment, declared mounts
Forbidden dependencies: planner/domain semantics, CLI adapters, report/analyzer ownership
Forbidden effects: network access unless declared, writes outside declared runtime roots
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runner --no-default-features`

## Why this crate exists
Owns controlled process/container execution for already planned and resolved tool invocations.

## Allowed dependencies
- Core, runtime, environment, infra, and policy support needed for backend invocation.
- No planner or domain semantics ownership.

## Allowed effects
- Controlled process/container spawning for declared invocation requests.
- Filesystem effects only under declared runtime paths.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
