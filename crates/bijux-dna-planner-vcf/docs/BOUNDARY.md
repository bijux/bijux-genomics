# bijux-dna-planner-vcf Boundary Contract

Owner: Planner
Scope: VCF deterministic plan assembly, reference resolution handoff, and explanation
Allowed inputs: VCF domain contracts, reference config views, stage contracts, registry views
Forbidden dependencies: runner backends, CLI adapters, environment probes, direct execution
Forbidden effects: process spawning, network access, product execution, generated config mutation
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-vcf --no-default-features`

## Why this crate exists
Builds VCF execution plans from domain/stage/reference contracts without owning runtime execution.

## Allowed dependencies
- Core, domain, DB-ref, stage-contract, and policy crates needed to assemble deterministic plans.
- No runner or CLI ownership.

## Allowed effects
- Pure deterministic planning and fixture-backed validation.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
