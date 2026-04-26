# bijux-dna-api Boundary Contract

Owner: API
Scope: Stable orchestration API over planner, runtime, environment, and analyzer contracts
Allowed inputs: typed request structs, planner outputs, runtime manifests, config views
Forbidden dependencies: CLI adapters as required runtime dependencies, ad hoc shell execution
Forbidden effects: direct process/container spawning, undeclared filesystem writes, network access
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-api --no-default-features`

## Why this crate exists
Provides the stable programmatic API surface for planning, dry-run, execution handoff, reporting,
environment checks, and operator failure classification.

## Allowed dependencies
- Planner, runtime, environment, runner, analyzer, and stage-contract crates required to compose API
  workflows.
- Domain crates only through typed contract surfaces.

## Allowed effects
- API-owned filesystem effects only when the called workflow declares output roots.
- No direct tool launching outside runtime/runner boundaries.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
