# bijux-dna-stages-vcf Boundary Contract

Owner: Stages
Scope: VCF stage invocation builders, path contracts, and observer parsers
Allowed inputs: VCF domain contracts, reference config views, fixture observations
Forbidden dependencies: CLI adapters, API orchestration, engine internals, planner orchestration
Forbidden effects: product execution outside declared fixture tests, network access, generated config writes
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-vcf --no-default-features`

## Why this crate exists
Owns VCF stage-level invocation and path/parsing contracts consumed by planners/runtime.

## Allowed dependencies
- Core, VCF domain, DB-ref, infra, policy, and testkit support.
- No command-surface or orchestration ownership.

## Allowed effects
- Pure invocation/path/observer contract construction and fixture-backed parsing.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
