# bijux-dna-environment-qa Boundary Contract

Owner: Environment QA
Scope: Environment smoke, artifact QA, and reproducibility evidence contracts
Allowed inputs: image catalogs, QA fixtures, run artifacts, declared cache roots
Forbidden dependencies: CLI adapters, planner selection logic, product pipeline ownership
Forbidden effects: undeclared network access, source mutation, writes outside QA output roots
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment-qa --no-default-features`

## Why this crate exists
Checks environment readiness and QA evidence without owning production runtime orchestration.

## Allowed dependencies
- Analyze, core, domain, environment, runtime, infra, policy, and testkit contracts needed for QA.
- No CLI or planner ownership.

## Allowed effects
- Controlled QA artifact reads/writes under declared roots.

## Notes
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.
