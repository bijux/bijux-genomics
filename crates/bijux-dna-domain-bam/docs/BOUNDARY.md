# bijux-dna-domain-bam Boundary Contract

Owner: Domain
Scope: BAM authored domain truth, IDs, vocabularies, contracts, and invariants
Allowed inputs: BAM domain source, typed core IDs, deterministic fixtures
Forbidden dependencies: runtime, runner, engine, API, CLI, planner orchestration
Forbidden effects: product execution, generated config writes, process spawning, network access
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-bam --no-default-features`

## Why this crate exists
Defines BAM domain truth and biological/contract invariants consumed by planners, stages, and
generated config views.

## Allowed dependencies
- Core IDs, policy contracts, and testkit fixtures needed to express domain semantics.
- No reverse-layer coupling to runtime or command surfaces.

## Allowed effects
- Pure deterministic computation over domain contracts and fixtures.

## Notes
The repository-level onboarding policy lives at `README.md`; child-repository work must apply that policy before changing this crate.
