# bijux-dna-engine Boundary

Owner: execution orchestration for an already planned `ExecutionGraph`.
Scope: order graph steps, invoke a caller-provided `Runner`, record engine truth,
and enforce engine-owned output contracts.
Allowed inputs: planned execution graphs, run layouts, cancellation tokens,
observability hooks, and runner responses.
Forbidden dependencies: planners, stage crates, domain crates, CLI adapters,
environment providers, and runner implementations.
Forbidden effects: process spawning, container selection, network access,
planning, domain interpretation, and ad hoc filesystem writes outside step run
artifacts and declared outputs.
Validation command:
`CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-engine --no-default-features`.

## Owns

- Sequential execution orchestration for a normalized `ExecutionGraph`.
- Cancellation checks before execution, between attempts, and after runner return.
- Retry and timeout policy application from `EngineConfig`.
- Engine event emission for step start, retry, step end, and artifact
  verification.
- Per-step `execution_record.json` persistence under `run_artifacts/`.
- Contract checks for declared outputs, required run artifacts, expected artifact
  IDs, and declared metrics envelopes.

## Does Not Own

- Workflow planning, stage selection, or tool parameter construction.
- Process spawning, container runtime behavior, backend probing, or tool
  execution adapters.
- Domain semantics for FASTQ, BAM, VCF, reports, metrics interpretation, or
  science policy.
- CLI/API request handling or environment provisioning.

## Allowed Dependencies

- `bijux-dna-core` for execution graph, artifact, run record, and identifier
  contracts.
- `bijux-dna-runtime` for `Runner`, `Invocation`, `RunnerResult`, run layout,
  and canonical recording helpers.
- `bijux-dna-infra` for filesystem helpers used by engine-owned recording and
  contract verification.

## Enforcement

- `tests/boundaries/effect_boundary.rs` rejects direct process/container effects.
- `tests/boundaries/architecture_tree.rs` locks source, docs, and test layout.
- `tests/contracts/architecture.rs` rejects a dependency edge to
  `bijux-dna-runner`.
- Repository policy tests reject planner, domain, stage, runner, and environment
  dependency edges.
