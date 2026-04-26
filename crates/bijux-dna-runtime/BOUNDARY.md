# bijux-dna-runtime Boundary Contract

Owner: Runtime
Scope: Run layout, execution context, telemetry, manifest, and runner handoff contracts
Allowed inputs: execution plans, runner responses, runtime profiles, declared run roots
Forbidden dependencies: CLI adapters, planner selection logic, domain semantics ownership
Forbidden effects: undeclared writes outside run layouts, hidden network access, direct planning
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runtime --no-default-features`

## Why this crate exists
Owns runtime contracts after planning and before/around runner backend execution.

## Allowed dependencies
- Core, infra, policy, and testkit support needed for runtime manifests and layout contracts.
- No CLI or planner-selection ownership.

## Allowed effects
- Writes only under declared run/layout roots.
- Process execution remains delegated through runner boundaries.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
