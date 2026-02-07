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

## Effect guarantees
- `cwd`: backend uses the working directory provided by the execution spec.
- `env`: only specified environment variables are injected; no implicit mutation.
- `mounts`: input mounts are read-only; output mounts are writable.
- `stdout/stderr`: captured verbatim and returned in `RunnerResult`.
- `exit_code`: nonzero exit codes are surfaced as failures in records.
See `tests/backend/backend_invariants.rs` for enforced invariants.

## Effects & determinism guarantees
Runner is the only allowed spawn boundary (plus allowlisted QA/CLI). See
`docs/EFFECTS.md`, `tests/backend/process_guardrail.rs`, and
`crates/bijux-policies/tests/surface/path_policies.rs`.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/backend/backend_invariants.rs`, `tests/replay/replay_contract.rs`, `tests/determinism/run_id_determinism.rs`, `tests/replay/replay_determinism.rs`.

## Where to start in code
- `src/runner_core.rs` for command execution primitives.
- `src/execute.rs` for step execution orchestration.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
