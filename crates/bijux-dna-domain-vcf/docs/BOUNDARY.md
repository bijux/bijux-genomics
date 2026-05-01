# bijux-dna-domain-vcf Boundary Contract

Owner: Domain
Scope: VCF authored domain truth, IDs, contracts, and downstream policy invariants
Allowed inputs: VCF domain source, typed IDs, deterministic fixtures
Forbidden dependencies: runtime, runner, engine, API, CLI, planner orchestration
Forbidden effects: product execution, generated config writes, process spawning, network access
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-vcf --no-default-features`

## Why this crate exists
Defines VCF domain truth and invariants consumed by planners, stages, and generated config views.

## Allowed dependencies
- Policy-neutral crate support required to express VCF contracts.
- No reverse-layer coupling to runtime or command surfaces.

## Allowed effects
- Pure deterministic computation over VCF contract inputs.

## Notes
The repository policy is rooted at `README.md`, and the repository coding
policy is rooted at `README.md`.
