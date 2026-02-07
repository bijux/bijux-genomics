# bijux-engine

## What this crate does
Pure orchestrator: executes ExecutionGraph via Runner, enforces contracts, records truth set.

## What it must not do (boundaries)
No process spawn or backend logic (docker/local).

## Role in the stack
Upstream: API. Downstream: runtime recorder + runner trait.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/ENGINE_MODEL.md`, `docs/DETERMINISM.md`, `docs/ERROR_TAXONOMY.md`, `docs/RECORDING_TRUTH_SET.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Per-step effective_config.json, tool_invocation.json, execution_record.json, metrics/stage_report when applicable.

## Effects & determinism guarantees
Orchestration only; effects happen in runner/runtime. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/effect_boundary.rs`, `tests/recording_completeness.rs`, `tests/replay_determinism.rs`, `tests/run_manifest.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
