# Architecture

This file is a brief map, not the full runner narrative. Detailed execution behavior lives in the focused docs listed below.

## Layout
- `lib.rs` exposes the public runner surface.
- `backend/` owns backend-specific execution concerns such as image resolution, execution specs, process execution, and replay.
- `step_runner/` owns step orchestration plus pure support modules for inputs, command templates, observer calls, identity, and artifacts.
- `command_runner.rs` owns low-level command invocation helpers.
- `repo_root.rs` owns repository-root lookup used by runtime resolution.

## Change rules
- Add new files only for distinct enduring execution concerns.
- Keep image resolution separate from process execution and replay behavior.
- Keep `step_runner/mod.rs` orchestration-focused and move pure helpers into companion modules.

## Pointers
- `INDEX.md` for the doc map.
- `BACKENDS.md`, `EXECUTION_SPEC.md`, and `REPLAY.md` for runner behavior.
- `CHANGE_RULES.md`, `FAILURES.md`, and `TESTS.md` for maintenance and verification.
