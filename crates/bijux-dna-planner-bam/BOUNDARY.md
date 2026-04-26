# bijux-dna-planner-bam Boundary Contract

Owner: Planner
Scope: BAM deterministic plan assembly, stage selection, and explanation
Allowed inputs: BAM domain contracts, stage contracts, pipeline profiles, registry views
Forbidden dependencies: runner backends, CLI adapters, environment probes, direct execution
Forbidden effects: process spawning, network access, product execution, generated config mutation
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-bam --no-default-features`

## Why this crate exists
Builds BAM execution plans from domain/stage contracts without owning runtime execution.

## Allowed dependencies
- Core, domain, pipeline, stage-contract, and stage crates needed to assemble deterministic plans.
- No runner or CLI ownership.

## Allowed effects
- Pure deterministic planning and fixture-backed validation.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
