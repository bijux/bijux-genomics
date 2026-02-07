# bijux-runner

## What this crate does
Executes command specs via local and docker backends and records execution results for runtime manifests.

## What it must not do (boundaries)
Must not plan graphs or parse tool outputs. It only executes and records. Planner/engine own selection and orchestration.

## Public API / entrypoints
Backend interfaces and execution entrypoints documented in `docs/BACKENDS.md` and `docs/EXECUTION_SPEC.md`.

## Key contracts it owns/consumes
Consumes `CommandSpec` and runtime recorder; emits execution records and stdout/stderr captures.

## Effects & determinism guarantees
This is the process-spawn boundary (except allowlisted QA/CLI). Deterministic replay is enforced in `tests/replay_determinism.rs`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/backend_invariants.rs`, `tests/replay_contract.rs`, `tests/determinism.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/BACKENDS.md`, `docs/REPLAY.md`, and `docs/SECURITY.md`.

## Artifacts / Contracts
Writes execution records and captures logs; no metrics parsing.

## Failure modes
Common failures include missing images or permission errors; see `docs/SECURITY.md` and `docs/REPLAY.md`.

## Stability
Backend invariants are stable and enforced by tests; see `docs/CHANGE_RULES.md`.
