# Architecture

This file is a brief map, not the full runner narrative. Detailed execution behavior lives in the focused docs listed below.

## Layout
- `lib.rs` is a thin crate root that exposes modules plus the stable root re-exports.
- `public_api/` owns the curated consumer-facing runner facade.
- `runner_driver.rs` owns the concrete `Runner` implementation used by higher layers.
- `backend/` owns backend-specific execution concerns such as image resolution, execution specs, process execution, replay, and backend-facing facade exports.
- `backend/facade.rs` keeps backend re-exports out of the backend root module.
- `backend/docker/facade.rs` keeps Docker-specific re-exports out of the Docker root module.
- `backend/docker/executor/` isolates Docker command-line assembly from container lifecycle observation.
- `backend/docker/executor/internal_contracts.rs` keeps executor coverage out of the execution implementation file.
- `backend/docker/image_resolution/` isolates Docker image availability policy from Apptainer registry lookup.
- `step_runner/` owns step orchestration plus support modules for contracts, runtime policy, container argument assembly, backend-specific execution, effects, records, inputs, observer calls, identity, and artifacts.
- `step_runner/internal_contracts.rs` keeps orchestration coverage out of the orchestrator file.
- `command_runner.rs` owns low-level command invocation helpers and delegates invocation identity to a companion module.
- `repo_root.rs` owns repository-root lookup used by runtime resolution.

## Change rules
- Keep `lib.rs` and backend root modules declarative; route curated exports through dedicated facade modules.
- Keep stable consumer entrypoints under `public_api/` and concrete runtime adapters under dedicated driver files.
- Add new files only for distinct enduring execution concerns.
- Keep image resolution separate from process execution and replay behavior.
- Keep Docker and Apptainer execution logic out of `step_runner/mod.rs`.
- Keep `step_runner/mod.rs` orchestration-focused and move support logic into companion modules such as `effects.rs`, `records.rs`, and `internal_contracts.rs`.

## Pointers
- `INDEX.md` for the doc map.
- `BACKENDS.md`, `EXECUTION_SPEC.md`, and `REPLAY.md` for runner behavior.
- `CHANGE_RULES.md`, `FAILURES.md`, and `TESTS.md` for maintenance and verification.
