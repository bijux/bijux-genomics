# Boundary

Owner: Runtime
Scope: Run layout, execution context, telemetry, manifest, and runner handoff contracts
Allowed inputs: execution plans, runner responses, runtime profiles, declared run roots
Forbidden dependencies: CLI adapters, planner selection logic, domain semantics ownership
Forbidden effects: undeclared writes outside run layouts, hidden network access, direct planning, process execution
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runtime --no-default-features`

`bijux-dna-runtime` owns the contracts that exist after planning and before or around runner execution. It defines run layouts, manifests, provenance, telemetry, observability contracts, runner handoff types, and append-only attempt transition persistence derived from execution evidence. It is not a backend runner, planner, CLI adapter, analyzer, or domain semantics owner.

## Allowed Responsibilities
- Create declared run-layout directories and runtime-owned files.
- Write canonical JSON and JSONL runtime artifacts.
- Define runner-facing contracts without executing commands.
- Load governed runtime registry metadata.
- Assemble provenance, telemetry, and observability records from declared inputs.
- Persist and reconstruct immutable benchmark attempt identities and sequence-addressed transitions.
- Validate only runtime-owned lifecycle entries inside benchmark attempt directories; sibling execution and evidence artifacts remain owned by their producing crates.

## Forbidden Responsibilities
- Spawning processes or invoking Docker/Apptainer directly.
- Selecting tools or planning stages.
- Owning CLI parsing, API transport, benchmark planning/reporting, or analyzer behavior.
- Writing outside declared run/layout roots.
- Performing hidden network access.
- Owning domain-specific BAM/FASTQ/VCF semantics.

## Enforcement
Policy scans and runtime boundary tests enforce effect limits, dependency shape, public API docs, and tree layout.
