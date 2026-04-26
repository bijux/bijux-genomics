# bijux-dna-domain-fastq Boundary Contract

Owner: Domain
Scope: FASTQ authored domain truth, IDs, contracts, bank contracts, and invariants
Allowed inputs: FASTQ domain source, governed bank metadata, typed core IDs, deterministic fixtures
Forbidden dependencies: runtime, runner, engine, API, CLI, planner orchestration
Forbidden effects: product execution, generated config writes, process spawning, network access
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-fastq --no-default-features`

## Why this crate exists
Defines FASTQ domain truth for bijux-dna: stage IDs, stage contracts, bank contracts,
parameter schemas/defaults, metric semantics, pipeline ordering, and invariant evaluation.

## Allowed dependencies
- Foundational workspace crates required to express domain data and parse governed assets.
- `bijux-dna-core` for identifiers and shared domain primitives.
- `bijux-dna-infra` for governed YAML/JSON parsing and content hashing.
- No reverse-layer coupling to planners, stages, runners, engines, containers, or CLIs.

## Allowed effects
- Read governed reference-bank, parameter, and test fixture files.
- Hash governed bank/reference files for provenance.
- No process spawning, network calls, container execution, mutable runtime state, or planner
  selection side effects.

## Notes
Boundary invariants are enforced by bijux-dna-policies contract tests.
The repository policy is rooted at `README.md`, and the repository coding
policy is rooted at `README.md`.
