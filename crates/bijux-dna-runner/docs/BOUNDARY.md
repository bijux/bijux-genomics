# Boundary

Owner: Runner
Scope: Backend process and container invocation boundary
Allowed inputs: explicit tool invocation requests, resolved runtime environment, declared mounts, execution manifests for replay
Forbidden dependencies: planner/domain semantics, CLI adapters, report/analyzer ownership
Forbidden effects: network access unless declared, writes outside declared runtime roots, tool selection, domain parsing
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runner --no-default-features`

`bijux-dna-runner` owns controlled Docker/Apptainer process execution for already planned and resolved tool invocations. It is an effect boundary, not a planner, parser, registry owner, CLI adapter, or analyzer.

## Allowed Runtime Responsibilities
- Build backend execution specs from typed tool execution contracts.
- Resolve declared container images through environment/runtime policy contracts.
- Spawn declared backend commands and capture stdout, stderr, and exit status.
- Write runner-owned execution artifacts under declared run/output roots.
- Replay existing execution manifests without executing tools.

## Forbidden Runtime Responsibilities
- Planning stages or selecting tools.
- Owning domain-specific parsing, metrics interpretation, reports, or CLI UX.
- Reading undeclared inputs or writing outside declared runtime paths.
- Enabling network access unless a runtime policy explicitly declares it.
- Reaching into engine internals or planner internals.

## Contract Changes
Boundary changes must update `DEPENDENCIES.md`, `EFFECTS.md`, `EXECUTION_SPEC.md`, and the matching runner boundary tests in the same reviewable change.
