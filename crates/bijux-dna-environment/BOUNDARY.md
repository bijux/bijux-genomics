# bijux-dna-environment Boundary Contract

Owner: Environment
Scope: Image catalog, runtime resolution, environment probing, and cache policy contracts
Allowed inputs: runtime profiles, image catalogs, platform definitions, declared cache roots
Forbidden dependencies: planner/domain semantics, CLI adapters, report/analyzer ownership
Forbidden effects: product execution, undeclared writes, network access outside declared prep workflows
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment --no-default-features`

## Why this crate exists
Resolves runtime and image environment facts without owning planning or product execution.

## Allowed dependencies
- Core, runtime model, infra, policy, and testkit support for environment contracts.
- No stage or domain semantics ownership.

## Allowed effects
- Controlled environment inspection and cache checks.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
