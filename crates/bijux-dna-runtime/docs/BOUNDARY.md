# Boundary

Owner: Runtime
Scope: Run layout, execution context, telemetry, manifest, and runner handoff contracts
Allowed inputs: execution plans, runner responses, runtime profiles, declared run roots
Forbidden dependencies: CLI adapters, planner selection logic, domain semantics ownership
Forbidden effects: undeclared writes outside run layouts, hidden network access, direct planning, process execution
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runtime --no-default-features`

`bijux-dna-runtime` owns the contracts that exist after planning and before or around runner execution. It defines run layouts, manifests, provenance, telemetry, observability contracts, and runner handoff types. It is not a backend runner, planner, CLI adapter, analyzer, or domain semantics owner.

## Allowed Responsibilities
- Create declared run-layout directories and runtime-owned files.
- Write canonical JSON and JSONL runtime artifacts.
- Define runner-facing contracts without executing commands.
- Load governed runtime registry metadata.
- Assemble provenance, telemetry, and observability records from declared inputs.

## Forbidden Responsibilities
- Spawning processes or invoking Docker/Apptainer directly.
- Selecting tools or planning stages.
- Owning CLI parsing, API transport, reports, benchmarks, or analyzer behavior.
- Writing outside declared run/layout roots.
- Performing hidden network access.
- Owning domain-specific BAM/FASTQ/VCF semantics.

## Enforcement
Policy scans and runtime boundary tests enforce effect limits, dependency shape, public API docs, and tree layout.
