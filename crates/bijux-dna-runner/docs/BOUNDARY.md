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
- Reject benchmark submission from mutable, dirty, or externally linked repository snapshots.
- Prepare and run exact-checksum external workers through immutable attempt-scoped Slurm evidence
  without interpreting their product outputs.

## Forbidden Runtime Responsibilities
- Planning stages or selecting tools.
- Owning domain-specific parsing, metrics interpretation, reports, or CLI UX.
- Reading undeclared inputs or writing outside declared runtime paths.
- Enabling network access unless a runtime policy explicitly declares it.
- Reaching into engine internals or planner internals.
- Executing `external-artifact://` plans through an ordinary local or container runner.

## Contract Changes
Boundary changes must update `DEPENDENCIES.md`, `EFFECTS.md`, `EXECUTION_SPEC.md`, and the matching runner boundary tests in the same reviewable change.

## Bundle transport

The runner owns publication and decoding of explicitly requested bundle transports. The benchmark model owns the persisted `BundleSidecar` schema. Bundle transport verifies ciphertext identity, declared backend, plaintext hash and byte count; it does not interpret the scientific payload. `backend/bundle` separates transport contracts, storage, the external age adapter, the deterministic mock adapter and their lifecycle.

Bundle recipient admission rejects empty, blank, padded or duplicate declarations before backend effects. On decode, ordered fingerprints must match the declared recipient strings; the deterministic mock envelope must also match those declarations. The age sidecar is unsigned: matching fingerprints are declaration consistency, not proof that the ciphertext targets those recipients. Age decryption authenticates its payload through the supplied identity, while the mock transport provides no confidentiality. No new dependency is required for this admission.
