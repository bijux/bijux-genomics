# Change Rules

These rules govern changes to engine orchestration, public API, recording truth,
effects, determinism, and dependency boundaries.

## Breaking Changes

A change is breaking when downstream crates, recorded truth, replay behavior, or
engine callers can observe a different contract.

Breaking changes include:

- Removing or renaming `Engine`, `EngineConfig`, `CancellationToken`,
  `EngineEvent`, `EngineHooks`, or `EngineError`.
- Changing `Engine::execute` inputs, return type, cancellation behavior, retry
  behavior, timeout behavior, or event ordering.
- Changing required per-step truth artifacts or their paths under
  `run_artifacts/`.
- Loosening contract checks for required run artifacts, declared outputs,
  expected artifact IDs, or metrics envelopes.
- Adding process, container, network, planning, or domain effects.
- Adding dependencies on planners, stages, domains, runner implementations,
  environment providers, API adapters, or CLI crates.
- Changing deterministic graph ordering or replay output semantics.

Breaking changes require explicit review, affected docs, updated tests, and
snapshot updates when snapshots cover the behavior.

## Non-Breaking Changes

The following are normally non-breaking when existing behavior remains stable:

- Adding a new `EngineEvent` variant while preserving existing variants and
  ordering.
- Adding optional `EngineConfig` fields with default behavior that preserves
  current execution.
- Adding stricter validation for states already invalid by documented contract.
- Clarifying docs without changing behavior.
- Adding tests that lock existing behavior.

## Required Updates

- Managed operations: update `docs/COMMANDS.md`.
- Source or test layout: update `docs/ARCHITECTURE.md`, `docs/TESTS.md`, and
  `tests/boundaries/architecture_tree.rs`.
- Dependency changes: update `docs/DEPENDENCIES.md` and dependency tests.
- Effects or recording truth: update `docs/EFFECTS.md` and contract tests.
- Public API changes: update `docs/PUBLIC_API.md`, `README.md`, and public API
  coverage.
- Determinism changes: update `docs/DETERMINISM.md` and determinism tests.
- Boundary changes: update `docs/BOUNDARY.md` and boundary tests.

## Verification

Run the narrowest suite for the changed surface. Before handoff, run:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-engine --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-engine --no-default-features
```
