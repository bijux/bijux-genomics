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
- Reverify and bind admitted operator-provided binaries or SIFs for explicit external-artifact
  plans before spawning them.
- Spawn declared backend commands and capture stdout, stderr, and exit status.
- Write runner-owned execution artifacts under declared run/output roots.
- Replay existing execution manifests without executing tools.
- Submit bounded Slurm batches without waiting, observe queue state once, collect
  terminal accounting, and request cancellation from typed scheduler identities.

## Forbidden Runtime Responsibilities
- Planning stages or selecting tools.
- Owning domain-specific parsing, metrics interpretation, reports, or CLI UX.
- Reading undeclared inputs or writing outside declared runtime paths.
- Enabling network access unless a runtime policy explicitly declares it.
- Reaching into engine internals or planner internals.
- Executing `external-artifact://` plans through an ordinary local or container runner.

## Contract Changes
Boundary changes must update `DEPENDENCIES.md`, `EFFECTS.md`, `EXECUTION_SPEC.md`, and the matching runner boundary tests in the same reviewable change.
