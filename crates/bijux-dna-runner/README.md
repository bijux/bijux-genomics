# bijux-dna-runner

## What this crate does
Container execution support for runner-facing commands, step orchestration, and replay.

## What it must not do (boundaries)
No planning or parsing; execution only.

## Role in the stack
Upstream: engine via runtime Runner. Downstream: runtime recorder.

## Public API / entrypoints
See `crates/bijux-dna-runner/docs/INDEX.md`, `crates/bijux-dna-runner/docs/BACKENDS.md`, `crates/bijux-dna-runner/docs/EXECUTION_SPEC.md`, `crates/bijux-dna-runner/docs/REPLAY.md`, `crates/bijux-dna-runner/docs/SECURITY.md`, `crates/bijux-dna-runner/docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Execution records and stdout/stderr captures.

## Artifacts / Contracts
See `crates/bijux-dna-runner/docs/EXECUTION_SPEC.md`, `crates/bijux-dna-runner/docs/BACKENDS.md`, and snapshots under `tests/snapshots/`.

## Effect guarantees
- `cwd`: backend uses the working directory provided by the execution spec.
- `env`: only specified environment variables are injected; no implicit mutation.
- `mounts`: input mounts are read-only; output mounts are writable.
- `stdout/stderr`: captured verbatim and returned in `RunnerResult`.
- `exit_code`: nonzero exit codes are surfaced as failures in records.
See `tests/backend/backend_invariants.rs` for enforced invariants.

## Effects & determinism guarantees
Runner is the only allowed spawn boundary (plus allowlisted QA/CLI). See
`crates/bijux-dna-runner/docs/EFFECTS.md`, `tests/backend/process_guardrail.rs`, and
`crates/bijux-dna-policies/tests/boundaries/surface/structure_layout/path_policies.rs`.

## How to run its tests
See `crates/bijux-dna-runner/docs/TESTS.md`. Golden tests: `tests/backend/backend_invariants.rs`, `tests/replay/replay_contract.rs`, `tests/determinism/run_id_determinism.rs`, `tests/replay/replay_determinism.rs`.

## Where to start in code
- `src/command_runner.rs` for command execution primitives.
- `src/backend/docker/` for backend-specific execution, image resolution, and replay.
- `src/step_runner/mod.rs` for step execution orchestration.

## Where the docs live
Start at `crates/bijux-dna-runner/docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-runner/docs/CHANGE_RULES.md`.
