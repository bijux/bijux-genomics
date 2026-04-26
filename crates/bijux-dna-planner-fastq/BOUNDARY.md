# bijux-dna-planner-fastq Boundary Contract

Owner: Planner
Scope: FASTQ deterministic plan assembly, tool selection, and explanation
Allowed inputs: FASTQ domain contracts, stage contracts, pipeline profiles, registry views
Forbidden dependencies: runner backends, CLI adapters, environment probes, direct execution
Forbidden effects: process spawning, network access, product execution, generated config mutation
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-fastq --no-default-features`

## Why this crate exists
Builds FASTQ execution plans from domain/stage contracts without owning runtime execution.

## Allowed dependencies
- Core, domain, pipeline, stage-contract, and stage crates needed to assemble deterministic plans.
- No runner or CLI ownership.

## Allowed effects
- Pure deterministic planning and fixture-backed validation.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
