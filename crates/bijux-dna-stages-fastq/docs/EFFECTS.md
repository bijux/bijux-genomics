# Effects

This crate is a stage-contract, observer, and metrics library. Production code
may read planned inputs and outputs, and may write explicit crate-owned observer
artifact JSON. It must not execute tools or own runtime orchestration.

## Allowed Production Effects

- Read existing FASTQ, gzip FASTQ, JSON, TSV, and text files referenced by a
  plan, artifact list, or observer fixture.
- Hash existing stage inputs for metrics provenance.
- Write explicit observer artifacts through `observer::artifacts`.
- Build deterministic invocation, report, warning, event, and metrics envelope
  values in memory.

## Forbidden Production Effects

- Process spawning, shell execution, container invocation, or tool installation.
- Network access.
- Planner orchestration, pipeline composition, runtime scheduling, retries, or
  cancellation.
- Environment setup or container image resolution.
- Tool selection or command-template construction.

## Test Effects

Tests may write temporary fixture copies and update snapshots only when the
explicit `UPDATE_CONTRACTS=1` workflow is used. Local cargo output must stay
under `artifacts/` when commands are invoked from the repository root.

## Enforcement

- `tests/boundaries/pipeline_guardrails.rs` rejects pipeline composition.
- `tests/boundaries/purity/purity.rs` rejects command construction and tool selection.
- `tests/boundaries/purity/architecture.rs` rejects process/container execution calls.
