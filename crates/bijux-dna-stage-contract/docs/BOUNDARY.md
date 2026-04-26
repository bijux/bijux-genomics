# Boundary

`bijux-dna-stage-contract` owns reusable planning contracts for stage plans,
execution plans, stage plugin payloads, and executor-readiness metadata.

## Belongs Here

- Stage-plan and execution-plan data structures.
- Planner-to-stage payload shapes.
- Artifact binding validation for planned handoffs.
- Executor registry labels and readiness badges used by policy checks.
- Deterministic schema fixtures and public contract snapshots.

## Does Not Belong Here

- Runtime manifests or executed-run records.
- Process spawning, container invocation, network access, or filesystem mutation.
- CLI parsing and command routing.
- Planner selection policy, API orchestration, runner backends, or environment QA.

## Dependency Boundary

Normal dependencies stay limited to:

- `bijux-dna-core` for typed IDs, artifact contracts, metrics envelopes, command
  specs, and canonical JSON helpers.
- `anyhow`, `serde`, `serde_json`, and `sha2` for contract validation,
  serialization, and deterministic plan hashing.

Dev dependencies may use `bijux-dna-policies`, `bijux-dna-testkit`, and
`walkdir` for guardrails, fixtures, and source scans.

## Validation

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --test boundaries --no-default-features
```
