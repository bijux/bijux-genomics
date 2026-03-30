# Architecture

## Goals
- Keep the crate root thin and explicit.
- Separate image resolution, command execution, and replay support.
- Keep `step_runner` focused on stage execution orchestration, with pure support logic isolated in companion modules.
- Preserve boundary tests and docs as the contract for future layout changes.

## Source tree

```text
src/
├── backend/
│   ├── docker/
│   │   ├── execution_spec.rs
│   │   ├── executor.rs
│   │   ├── image_resolution.rs
│   │   ├── mod.rs
│   │   └── replay.rs
│   ├── kinds.rs
│   └── mod.rs
├── command_runner.rs
├── lib.rs
├── repo_root.rs
└── step_runner/
    ├── artifacts.rs
    ├── command_template.rs
    ├── identity.rs
    ├── inputs.rs
    ├── mod.rs
    └── observer.rs
```

## Responsibilities

### `lib.rs`
- Exposes the public runner facade.
- Maps backend execution results into crate-level result types.

### `backend/`
- Owns backend-specific execution mechanics.
- `docker/execution_spec.rs` builds runtime execution specs from registry and platform data.
- `docker/image_resolution.rs` resolves concrete images and local availability.
- `docker/executor.rs` executes container plans and inspects runtime state.
- `docker/replay.rs` owns replay-oriented backend behavior.

### `step_runner/`
- `mod.rs` coordinates step execution and result assembly.
- `inputs.rs` handles bind roots and container input mapping.
- `command_template.rs` rewrites host paths into container paths.
- `observer.rs` runs lightweight observer commands.
- `identity.rs` owns hashing and execution identity helpers.
- `artifacts.rs` writes minimal run artifact payloads.

### Cross-cutting support
- `command_runner.rs` owns low-level command invocation helpers.
- `repo_root.rs` resolves repository-local paths used by runtime lookup.

## Change rules
- Add new files only when they own a distinct execution concern.
- Keep backend image selection separate from backend process execution.
- Keep `step_runner/mod.rs` orchestration-focused; move pure support logic into companion modules.
- Update this document and the boundary tree contract together when the layout changes intentionally.
