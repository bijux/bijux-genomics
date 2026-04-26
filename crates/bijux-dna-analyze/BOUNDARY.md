# bijux-dna-analyze Boundary Contract

Owner: Analyze
Scope: Report, analytics, provenance, and benchmark-result interpretation over produced artifacts
Allowed inputs: run manifests, stage reports, benchmark reports, deterministic fixture data
Forbidden dependencies: runner backends, engine internals, CLI adapters, planner selection logic
Forbidden effects: product execution, generated config mutation, network access
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-analyze --no-default-features`

## Why this crate exists
Turns completed run and benchmark artifacts into deterministic reports, summaries, rankings, and
contract diagnostics.

## Allowed dependencies
- Core, runtime, pipeline, domain, benchmark, policy, infra, and testkit contracts needed to read
  and validate produced artifacts.
- No execution-layer ownership.

## Allowed effects
- Read produced artifacts and fixtures.
- Write declared report outputs only.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
