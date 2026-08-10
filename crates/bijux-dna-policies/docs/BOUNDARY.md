# Boundary

Owner: Policies
Scope: pure policy decisions plus deterministic repository policy checks
Allowed inputs: repository files, policy fixtures, documentation, Cargo manifests, and governed config files
Forbidden dependencies: product execution crates as runtime dependencies, runner backends, domain-specific orchestration ownership
Forbidden effects: source mutation, generated config rewrites, snapshot rewrites, network access, process execution from production source
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --no-default-features`

`bijux-dna-policies` owns governance logic for this repository. It may inspect the workspace and report contract violations. It must not become a product runtime, command runner, domain semantic owner, or generated-output writer.

## Allowed Runtime Responsibilities
- Curate policy assertion macros.
- Run guardrails against crate source trees.
- Render policy diagnostics through a stable WHAT/WHY/HOW/MORE format.
- Scan Rust source and static repository files deterministically.
- Provide stable guardrail configuration data.
- Derive typed qualification claims and portable report summaries from admitted evidence facts.
- Validate publication freshness and typed container security evidence without executing scanners.
- Derive and admit checksum-bound release publications without performing publication I/O.

## Allowed Test Responsibilities
- Inspect workspace dependency graphs and manifests.
- Validate repository docs, ownership, boundaries, snapshots, and fixtures.
- Exercise model crates only enough to prove policy contracts.

## Forbidden Runtime Responsibilities
- CLI parsing and runtime subcommand ownership.
- Tool execution, process spawning, container control, or network access.
- Pipeline, planner, stage, runtime, analyzer, API, or database semantics.
- Writing files, blessing snapshots, or mutating generated configuration.

## Contract Changes
Boundary changes must update this file, `DEPENDENCIES.md`, `EFFECTS.md`, and the relevant policy tests in the same reviewable change.
