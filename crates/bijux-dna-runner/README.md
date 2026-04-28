# bijux-dna-runner

## What this crate does
`bijux-dna-runner` is the controlled execution boundary for already-planned DNA workflow steps. It turns typed execution specs into Docker or Apptainer process calls, captures stdout, stderr, exit status, and runner artifacts, and can replay existing execution records without running tools again.

## What it must not do (boundaries)
This crate must not plan stages, select tools, parse domain data, own CLI UX, produce analyzer reports, or reach into engine/planner internals. It receives resolved execution intent and performs only the declared runner effect.

## Role in the stack
Upstream callers use the `bijux-dna-runtime::Runner` contract and runner facade exports. Downstream effects are process execution, artifact collection, and execution-record replay under declared runtime roots.

## Public API / entrypoints
Use `bijux_dna_runner::api::*` for stable consumer-facing entrypoints. The crate root also exports `DockerRunner` for the concrete runtime adapter. See `docs/PUBLIC_API.md` for the full facade contract and `docs/COMMANDS.md` for the command inventory this crate may manage.

## Key contracts it owns/consumes
The runner owns backend execution specs, command invocation identity, stdout/stderr capture, exit mapping, runner artifacts, replay verification, and the dependency/effect boundary around those responsibilities.

## Artifacts / Contracts
See `docs/EXECUTION_SPEC.md` for execution records and backend invariants, `docs/DETERMINISM.md` for replay and identity guarantees, and `docs/EFFECTS.md` for allowed side effects.

## Effect guarantees
- `cwd`: backend uses the working directory provided by the execution spec.
- `env`: only specified environment variables are injected; no implicit mutation.
- `mounts`: input mounts are read-only; output mounts are writable.
- `stdout/stderr`: captured verbatim and returned in `RunnerResult`.
- `exit_code`: nonzero exit codes are recorded as failed execution outcomes.
See `tests/boundaries/backend/backend_invariants.rs` and `tests/schemas/docs_backend_invariants.rs` for enforced invariants.

## Effects & determinism guarantees
Runner is the only crate-local process execution boundary. Replay does not spawn tools, pull images, or mutate execution inputs. See `docs/EFFECTS.md`, `docs/DETERMINISM.md`, `tests/boundaries/backend/process_guardrail.rs`, and `tests/boundaries/backend/network_guardrail.rs`.

## How to run its tests
Run:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runner --no-default-features
```

See `docs/TESTS.md` for suite ownership and failure meaning.

## Where to start in code
- `src/public_api/mod.rs` for the stable consumer-facing runner surface.
- `src/runner_driver/` for the `Runner` implementation used by higher layers.
- `src/command_runner.rs` for command execution primitives and invocation identity wiring.
- `src/backend/` for backend kinds and backend-facing facade exports.
- `src/step_runner/` for Docker/Apptainer orchestration, effects, records, identity, inputs, and artifacts.

## Where the docs live
The crate root has only this `README.md`. All other docs live in `docs/`; start at `docs/INDEX.md`.

## Failure modes
Runtime failures surface through backend process errors, missing images, permissions, timeouts, nonzero exits, or replay artifact mismatches. See `docs/EXECUTION_SPEC.md` and `docs/DETERMINISM.md`.

## Stability
Contract changes must update the relevant docs and boundary tests in the same reviewable change.

## Repository Policy
This crate follows repository governance documentation. `/Users/bijan/bijux/bijux-genomics/README.md`,
`README.md`, and `README.md`; re-read
those files before editing this child repository or making commits.
