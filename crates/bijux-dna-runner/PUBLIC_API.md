# bijux-dna-runner Public API

Public modules exported from `src/lib.rs`:
- backend
- command_runner
- public_api
- step_runner

Root re-exports from `src/lib.rs`:
- `api`
- `DockerRunner`

Stable facade exports under `src/public_api/mod.rs`:
- `BackendKind`
- `build_tool_execution_spec`
- `parse_mem_to_mb`
- `replay_run`
- `invocation_hash`
- `run_command`
- `run_command_with_context`
- `CommandOutputV1`
- `execute_observer_command`
- `execute_step`
- `StageResultV1`
