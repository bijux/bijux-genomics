# bijux-dna-stages-fastq Boundary Contract

Owner: Stages
Scope: FASTQ stage invocation builders, output contracts, and observer parsers
Allowed inputs: FASTQ domain contracts, shared stage contracts, fixture observations
Forbidden dependencies: CLI adapters, API orchestration, engine internals, planner orchestration
Forbidden effects: product execution outside declared fixture tests, network access, generated config writes
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-fastq --no-default-features`

## Why this crate exists
Owns FASTQ stage-level invocation and observer contracts consumed by planners/runtime.

## Allowed dependencies
- Core, FASTQ domain, infra, runtime models, stage-contract, policy, and testkit support.
- No command-surface or orchestration ownership.

## Allowed effects
- Pure invocation/observer contract construction and fixture-backed parsing.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
