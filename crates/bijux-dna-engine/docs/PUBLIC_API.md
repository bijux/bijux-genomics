# Public API

`bijux-dna-engine` keeps a narrow public surface. The crate root re-exports the
stable surface curated by `src/public_api/stable_surface.rs`.

## Module Inventory

- public_api

## Stable Root Exports

- `Engine` - execution entrypoint for a planned `ExecutionGraph`; validates
  engine policy and the caller-provided `RunLayout` before invoking a runner.
- `EngineConfig` - timeout, retry, deterministic scheduling, and parallelism
  policy input.
- `CancellationToken` - cooperative cancellation state shared with callers.
- `EngineEvent` - event enum emitted through hooks.
- `EngineHooks` - callback trait for observing engine events.
- `EngineError` - engine-owned error taxonomy for validation and contract
  failures.

## Extension Rules

- Add stable items through `src/public_api/stable_surface.rs`.
- Keep `src/lib.rs` thin: module declarations plus curated re-exports only.
- Do not expose executor internals, graph preparation internals, recording
  writers, or contract-check modules as public API.
- New callable engine operations must be listed in `docs/COMMANDS.md`.
- Public API changes must update this file, `README.md`, and the closest
  contract or schema test in the same change set.

## Execution Preconditions

`Engine::execute` requires a canonical `RunLayout` whose `run_dir`,
`stages_dir`, and `summary_dir` already exist. The engine does not create the
run layout; callers should use `bijux-dna-runtime::run_layout::create_run_layout`
or an equivalent runtime-owned setup before calling the engine.

## Enforcement

- `tests/boundaries/architecture_tree.rs` locks `src/public_api/`.
- `tests/contracts/execution_orchestration_contracts.rs` covers public
  `Engine`, `EngineConfig`, `CancellationToken`, `EngineEvent`, `EngineHooks`,
  and run-layout precondition behavior.

## Stability Tiers

- Stable: the `Engine` root surface and the types listed under Stable Root Exports.
- Experimental: new engine entrypoints remain experimental until they are added to `src/public_api/stable_surface.rs` and this document.
- Internal: executor internals, graph preparation helpers, recording writers, and any module outside the curated public surface.
