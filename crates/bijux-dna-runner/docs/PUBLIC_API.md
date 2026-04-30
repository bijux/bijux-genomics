# Public API

`bijux-dna-runner` exposes a small runner facade plus backend and step-runner modules needed by current workspace callers. Internal support modules remain private.

## Public Modules
- `backend`
- `command_runner`
- `public_api`
- `step_runner`

## Root Exports
- `api`
- `DockerRunner`
- `LocalRunner`

## Facade Exports
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

## Stability Rules
- Prefer `bijux_dna_runner::api::*` for consumer-facing use.
- `DockerRunner` and `LocalRunner` are the concrete runtime adapters exported at the crate root for higher layers.
- Backend modules may expose backend contracts, but planning and CLI concerns must stay outside this crate.

## Source Authorities
- `src/lib.rs` controls public module visibility and root re-exports.
- `src/public_api/stable_surface.rs` curates facade exports.
- `src/backend/stable_surface.rs` and `src/backend/docker/stable_surface.rs` curate backend exports.

## Stability Tiers

- Stable: `api`, `DockerRunner`, `LocalRunner`, and the documented facade exports above.
- Experimental: backend-specific additions are experimental until they are listed under Facade Exports or Root Exports in this file.
- Internal: support modules and any runner/backend helper not re-exported through the documented stable surfaces.
