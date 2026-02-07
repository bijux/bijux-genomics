# bijux-runner

## What this crate does
Execution backends (local/docker) that run CommandSpec and capture outputs.

## What it must not do (boundaries)
No planning or parsing; execution only.

## Role in the stack
Upstream: engine via runtime Runner. Downstream: runtime recorder.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/BACKENDS.md`, `docs/EXECUTION_SPEC.md`, `docs/REPLAY.md`, `docs/SECURITY.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Execution records and stdout/stderr captures.

## Effects & determinism guarantees
Process spawn boundary (plus allowlisted QA/CLI). See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/backend_invariants.rs`, `tests/replay_contract.rs`, `tests/determinism.rs`, `tests/replay_determinism.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
