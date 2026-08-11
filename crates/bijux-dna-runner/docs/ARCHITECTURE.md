# Architecture

`bijux-dna-runner` is a library crate with one root README and all supporting docs in `docs/`. Source files are organized around a narrow execution boundary: stable facade exports, command execution helpers, backend kind contracts, repo-root lookup, concrete runner driver behavior, and step orchestration.

## Root Layout
- `Cargo.toml` declares runner dependencies and feature gates.
- `README.md` is the only root documentation file.
- `docs/` contains the 10 authoritative crate docs.
- `src/` contains the library implementation.
- `tests/` contains boundary, contract, determinism, guardrail, schema,
  semantic, and support coverage.

## Layout
- `lib.rs` is a thin crate root that exposes modules plus the stable root re-exports.
- `public_api/` owns the curated consumer-facing runner facade, with `stable_surface.rs` carrying the stable export contract.
- `runner_driver/` owns the concrete `Runner` implementation used by higher layers, with artifact collection split into a dedicated companion file.
- `backend/` owns backend kind contracts and backend-facing facade exports.
- `backend/slurm/` owns bounded non-waiting scheduler submission, one-shot queue
  observation, terminal accounting collection, cancellation, fail-closed parsing, and immutable
  benchmark-source admission before scheduler material is persisted.
- `backend/slurm/qualification/` owns immutable container qualification preparation,
  intent-first submission, disconnect-safe recovery, one-shot observation, terminal scheduler
  evidence, process execution, and cancellation. Container contract admission remains in
  `bijux-dna-containers`; probe planning and verdict evaluation remain in
  `bijux-dna-containers-qa`.
- `backend/stable_surface.rs` keeps stable backend exports out of the backend root module.
- `backend/facade.rs` keeps backend re-exports out of the backend root module.
- `step_runner/` owns step orchestration plus support modules for contracts, runtime policy, dispatch, container argument assembly, backend-specific execution, effects, records, inputs, observer calls, identity, and artifacts.
- `step_runner/internal_contracts.rs` keeps orchestration coverage out of the orchestrator file.
- `command_runner.rs` owns low-level command invocation helpers and delegates site execution-binding
  verification, invocation identity, command-line rendering, and output contracts to companion
  modules.
- `repo_root/` owns repository-root lookup used by runtime resolution, with env override lookup split from repository detection.

## Backend Flow
1. A typed command spec and runtime policy arrive from upstream.
2. `step_runner` builds Docker or Apptainer arguments from typed contracts.
3. `command_runner` executes the declared command and captures output.
4. `step_runner` normalizes the outcome, records artifacts, and returns a stable result.
5. Replay reads manifests and artifacts only; it does not spawn backend commands.

Slurm execution is cycle-driven: submission returns a job handle immediately,
later status calls query `squeue` once, and jobs absent from the queue are queried
once through `sacct`. Missing accounting remains explicit and is not success.

## Change rules
- Keep `lib.rs` and backend root modules declarative; route curated exports through dedicated facade modules.
- Keep stable consumer entrypoints under `public_api/` and concrete runtime adapters under dedicated driver files.
- Add new files only for distinct enduring execution concerns.
- Keep stable re-export contracts in `stable_surface.rs` files instead of root modules.
- Keep runtime resolution separate from process execution and replay behavior.
- Keep Docker and Apptainer execution logic out of `step_runner/mod.rs`.
- Keep `step_runner/mod.rs` orchestration-focused and move support logic into companion modules such as `effects.rs`, `records.rs`, and `internal_contracts.rs`.

## Test Layout
- `tests/boundaries/backend/` protects backend invariants, fixture parity,
  invocation identity, process effects, and network guardrails.
- `tests/contracts/` exercises backend contracts through the public runner
  surface.
- `tests/determinism/replay/` keeps replay contracts separate from replay
  determinism checks.
- `tests/schemas/` locks backend invariant documentation.
- `tests/semantics/` owns Docker parsing semantics.
- `tests/support/workspace_paths.rs` contains shared workspace-root helpers only.

## Dependency Direction
Runner may depend on core, runtime, container, container-QA, and low-level infrastructure
contracts needed to execute declared backend commands. It must not plan workflows, select domain
tools, own qualification verdicts, own CLI behavior, or depend on API/engine orchestration.

## Pointers
- `INDEX.md` for the doc map.
- `EXECUTION_SPEC.md` and `DETERMINISM.md` for runner behavior.
- `BOUNDARY.md`, `DEPENDENCIES.md`, and `EFFECTS.md` for architectural limits.
- `TESTS.md` for maintenance and verification.
