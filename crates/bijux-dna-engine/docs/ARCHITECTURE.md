# Architecture

`bijux-dna-engine` executes a fully formed graph through a caller-provided
runtime `Runner`. It coordinates execution; it does not plan workflows, select
tools, interpret domain semantics, or perform backend effects directly.

## Source Map

- `src/engine_driver.rs` owns `Engine::execute` and run-layout validation.
- `src/engine_config/` owns engine execution policy and graph application.
- `src/control/` owns cancellation contracts and state transitions.
- `src/executor/` owns graph normalization, step ordering, runner invocation,
  retry handling, recording, and contract verification.
- `src/observability/` owns engine events and hook contracts.
- `src/public_api/` exposes the stable engine surface.

## Executor Map

- `executor/graph/` prepares deterministic execution order.
- `executor/step_execution/` coordinates runner calls and step lifecycle.
- `executor/contracts/` verifies outputs, metrics, and run artifacts.
- `executor/recording/` writes execution records and manifest data.

## Test Map

- `tests/boundaries.rs` checks source layout, dependencies, docs placement, and
  effect boundaries.
- `tests/contracts.rs` checks execution orchestration and recording contracts.
- `tests/determinism.rs` checks replay and manifest stability.
- `tests/support/` contains reusable execution fixtures.

## Boundaries

The engine accepts planned `ExecutionGraph` values and canonical runtime
layouts. It must not depend on domain crates, planner crates, stage crates, or
runner backend implementations.

## Command Inventory

`docs/COMMANDS.md` lists the library operations this crate manages. Keep it in
sync with `Engine::execute` and execution contract tests.
