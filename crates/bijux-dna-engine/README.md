# bijux-dna-engine

## What this crate does
Pure orchestrator: executes `ExecutionGraph` via `Runner`, enforces contracts, and records the per-step truth set.

## Engine lifecycle (minimal)
```text
plan -> validate -> execute (Runner) -> record -> enforce contract
```

## Executor responsibilities
`src/executor.rs` owns:
- graph validation + scheduling decisions (including deterministic ordering),
- invoking the `Runner` for each step,
- emitting the per-step truth set into `run_artifacts/`.

The truth set per step is:
- `effective_config.json`
- `tool_invocation.json`
- `metrics.json`
- `stage_report.json`

## What it must not do (boundaries)
No process spawn or backend logic (docker/local). See `crates/bijux-dna-engine/docs/EFFECT_BOUNDARY.md`.

## Role in the stack
Upstream: API. Downstream: runtime recorder + runner trait.

## Public API / entrypoints
Start at `crates/bijux-dna-engine/docs/INDEX.md`. Key docs:
- `crates/bijux-dna-engine/docs/ENGINE_CONTRACT.md`
- `crates/bijux-dna-engine/docs/ERRORS.md`
- `crates/bijux-dna-engine/docs/EFFECTS.md`

## Key contracts it owns/consumes
Per-step effective_config.json, tool_invocation.json, execution_record.json, metrics/stage_report when applicable.

## Artifacts / Contracts
See `crates/bijux-dna-engine/docs/ENGINE_CONTRACT.md` and snapshots under `tests/snapshots/`.

## Effects & determinism guarantees
Orchestration only; effects happen in runner/runtime. See `crates/bijux-dna-engine/docs/EFFECT_BOUNDARY.md`, `crates/bijux-dna-engine/docs/EFFECTS.md`,
and `crates/bijux-dna-engine/docs/DETERMINISM.md` plus the golden tests below.

## How to run its tests
See `crates/bijux-dna-engine/docs/TESTS.md`. Golden tests: `tests/contracts/effect_boundary.rs`, `tests/recording/recording_completeness.rs`, `tests/determinism/replay_determinism.rs`, `tests/recording/run_manifest.rs`.

## Where the docs live
Start at `crates/bijux-dna-engine/docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-engine/docs/CHANGE_RULES.md`.

## Why engine is not planner
The engine executes a fully-formed `ExecutionGraph`; planners build that graph from domain inputs and policies. Keeping the engine planner-free prevents domain drift and ensures planners can evolve without changing orchestration semantics (see planners under `crates/bijux-dna-planner-fastq/ and crates/bijux-dna-planner-bam/`).
