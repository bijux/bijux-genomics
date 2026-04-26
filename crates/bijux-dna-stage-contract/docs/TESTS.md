# Tests

## Intent
The stage-contract test tree protects the reusable stage schema contract. Tests stay grouped by
contract intent, with taxonomy documented here instead of in `tests/README.md` placeholder files.

## Suite Entrypoints
- `tests/boundaries.rs`: crate ownership, no-execution guardrails, and tree contracts.
- `tests/contracts.rs`: API, metadata, stage-instance, and versioning behavior.
- `tests/determinism.rs`: fixture and stable-output checks.
- `tests/schemas.rs`: public type and schema snapshots.
- `tests/guardrails.rs`: shared policy guardrail smoke coverage.

## Suite Directories
- `tests/boundaries/guardrails/`: no process execution and layout ownership checks.
- `tests/contracts/versioning/`: semantic-version and SSOT versioning contracts.
- `tests/determinism/`: fixture stability checks.
- `tests/schemas/schema/`: public type, docs, and schema snapshot checks.

## No-Execution Boundary
The no-execution scan forbids process spawning and runtime effects in this crate. Stage execution
belongs in runner/runtime crates, not in the shared contract model.

## Commands
Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --no-default-features
```

## Failure Modes
- Boundary failures mean the contract crate gained behavior, effects, or undocumented layout drift.
- Contract failures mean public metadata, instance identity, or versioning behavior changed.
- Determinism failures mean fixtures or stable output changed.
- Schema failures mean the public contract shape or snapshots changed.

## Testkit Patterns
See `crates/bijux-dna-testkit/docs/USAGE.md` for shared fixture and snapshot helpers.
