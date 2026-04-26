# Effects

`bijux-dna-runner` is an effect boundary. It may spawn declared backend commands and write declared runner artifacts, but it must keep those effects explicit, typed, and test-covered.

## Allowed Effects
- Spawn `docker run` and `apptainer exec` through typed execution specs.
- Spawn declared observer commands through `execute_observer_command`.
- Spawn declared low-level commands through `run_command` and `run_command_with_context`.
- Read declared inputs and execution manifests.
- Write runner-owned records, stdout/stderr captures, and artifacts under declared run/output roots.
- Create temporary paths through `bijux-dna-infra` helpers.

## Effect Codes
Runner errors use these source-level effect codes:

- `filesystem`
- `command_spawn`
- `container_lifecycle`
- `telemetry_write`

## Forbidden Effects
- No CLI parsing or command discovery.
- No planner, engine, analyzer, or report effects.
- No writes outside declared runtime roots.
- No network access by default.
- No privileged containers or host-root mounts.
- No secret logging.
- Replay must not spawn processes, pull images, or mutate input artifacts.

## Environment Rules
- Inject only environment variables declared in the execution spec.
- `BIJUX_ALLOW_NETWORK` is the explicit opt-in for backend network access.
- Do not mutate process-wide environment state for callers.
- Treat runtime policy as input; this crate does not invent policy.

## Validation
- `tests/boundaries/backend/process_guardrail.rs` confines process spawning.
- `tests/boundaries/backend/network_guardrail.rs` keeps network behavior explicit.
- `tests/boundaries/command_inventory.rs` keeps managed command families documented.

## Failure modes
- Forbidden process, network, or filesystem behavior fails boundary tests.
- Runtime command failures are recorded as execution failures; see `EXECUTION_SPEC.md`.
