# bijux-engine

## What this crate does
Pure orchestrator: executes an `ExecutionGraph` through a `Runner`, enforces contracts, and records the truth set per step.

## What it must not do (boundaries)
Must not spawn processes or know about docker/local execution. All execution effects live in runner.

## Public API / entrypoints
`Engine::execute` is the single entrypoint. See `docs/ENGINE_MODEL.md` and `docs/RECORDING_TRUTH_SET.md`.

## Key contracts it owns/consumes
Consumes core contracts (`ExecutionGraph`, `RunManifest`) and runtime recorder/runner traits. Produces step records and manifests.

## Effects & determinism guarantees
Orchestration only; determinism is enforced by `tests/replay_determinism.rs` and `tests/recording_completeness.rs`. See `docs/DETERMINISM.md`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/effect_boundary.rs`, `tests/recording_completeness.rs`, `tests/replay_determinism.rs`, `tests/run_manifest.rs`.

## Where the docs live
Start at `docs/INDEX.md`. See `docs/ERROR_TAXONOMY.md` and `docs/RECORDING_TRUTH_SET.md`.

## Artifacts / Contracts
Emits per-step `effective_config.json`, `tool_invocation.json`, `execution_record.json`, and optional `metrics.json` / `stage_report.json`.

## Failure modes
Contract enforcement failures include step id, artifact id, and path; see `docs/ERROR_TAXONOMY.md`.

## Stability
Orchestration semantics are stable; changes require updating determinism tests and `docs/CHANGE_RULES.md`.
