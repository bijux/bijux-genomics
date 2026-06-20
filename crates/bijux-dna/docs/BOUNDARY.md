# bijux-dna Boundary Contract

Owner: CLI
Scope: User-facing command adapter over stable API, registry, environment, and operator helpers
Allowed inputs: CLI arguments, current working directory, repository configs, explicit environment variables, API responses
Forbidden dependencies: engine internals, runner internals, runtime internals, and undeclared internal crates that bypass owned benchmark and fixture workflows
Forbidden effects: undeclared writes, undeclared network access, process spawning, hidden runtime mutation
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --no-default-features`

## Why this crate exists
`bijux-dna` is the operator command surface. It turns command-line input into typed API calls,
renders human/JSON output, and exits with categorized operator failures.

## What this crate owns
- CLI parser types and command names.
- Command routing and process-free CLI entrypoints for integration tests.
- Terminal/JSON rendering at the CLI boundary.
- Help text, command snapshots, and operator-facing error categorization.
- Thin adapters for environment, registry, corpus, benchmark, status, BAM, FASTQ, and VCF commands.
- Benchmark-readiness validation, governed fixture regeneration, and local smoke orchestration that this crate publishes as operator-facing benchmark surfaces.

## What this crate must not own
- Scientific domain truth, stage catalogs, or tool semantics.
- Planner selection policy.
- Stage execution or runner backend behavior.
- Hidden network access or shell/container spawning.
- Workspace dependency decisions that belong to architecture policy.

## Allowed dependencies
- `bijux-dna-api` for stable planning, runtime, report, environment, and domain-facing contracts.
- `bijux-dna-infra` for declared filesystem helpers.
- `bijux-dna-core`, domain, planner, stage-contract, and selected stage crates when the CLI owns benchmark-readiness validation, fixture truth generation, or local smoke evidence for governed outputs.
- Explicit data access adapters when the CLI command owns that operator workflow, such as ENA corpus
  materialization.
- CLI support libraries for parsing, serialization, rendering, and deterministic archives.
- Debug/operator helper crates only when explicitly justified by command ownership.

## Forbidden dependency shape
The CLI must not directly depend on engine crates, runner crates, runtime internals, or unrelated
internal crates to run science logic. Direct domain, planner, stage-contract, and selected stage
crate edges are allowed only for benchmark-owned validation, fixture, and local smoke commands
whose durable outputs are governed by this crate; broader runtime execution paths must still move
behind `bijux-dna-api` or a smaller owning crate before the CLI calls them.

## Allowed effects
- Read repository configs and governed domain/assets files through declared paths.
- Write command outputs requested by flags or documented command contracts.
- Create deterministic observability log packs from declared run artifacts.

## Notes
The family-level contract is indexed in
[docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md](../../../docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md).
