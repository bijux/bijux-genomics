# bijux-dna-bench Boundary Contract

Owner: Benchmark
Scope: Benchmark model orchestration, scoring support, and benchmark contract helpers
Allowed inputs: benchmark configs, analyzer outputs, domain fixtures, runtime summaries
Forbidden dependencies: CLI adapters, runner backends as required runtime deps, direct tool execution
Forbidden effects: product execution, network access, undeclared writes outside benchmark roots
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --no-default-features`

## Why this crate exists
Owns benchmark contract helpers and scoring surfaces without launching product workflows.

## Allowed dependencies
- Analyze, core, domain, model, runtime, infra, policy, and testkit contracts needed for benchmark
  interpretation.

## Allowed effects
- Deterministic reads/writes under declared benchmark artifacts.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
