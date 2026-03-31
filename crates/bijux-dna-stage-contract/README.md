# bijux-dna-stage-contract

## What this crate does
Minimal planning contract types for stage plans and plugins.

## What this crate is NOT
- Not a runtime crate (no execution artifacts).
- Not a runner crate (no process execution).
- Not a docker/container crate.

## What it must not do (boundaries)
No execution details or runner/env concepts.

## Role in the stack
Upstream: core IDs. Downstream: planners/engine.

## Public API / entrypoints
See `crates/bijux-dna-stage-contract/docs/INDEX.md`, `crates/bijux-dna-stage-contract/docs/CONTRACT.md`, `crates/bijux-dna-stage-contract/docs/PUBLIC_API.md`, `crates/bijux-dna-stage-contract/docs/SCHEMAS.md`,
`crates/bijux-dna-stage-contract/docs/VERSIONING.md`, `crates/bijux-dna-stage-contract/docs/MINIMALITY.md`, `crates/bijux-dna-stage-contract/docs/CHANGE_RULES.md`.

## Public types
- `ExecutionPlan`
- `StagePlan`
- `StageSpecRef`
- `PluginSpec`

## Key contracts it owns/consumes
Plan JSON shapes and fixtures.

## Artifacts / Contracts
See `crates/bijux-dna-stage-contract/docs/CONTRACT.md`, `crates/bijux-dna-stage-contract/docs/SCHEMAS.md`, and snapshots under `tests/snapshots/`.

## Effects & determinism guarantees
Pure contract types; deterministic serialization. See `crates/bijux-dna-stage-contract/docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `crates/bijux-dna-stage-contract/docs/TESTS.md`. Golden tests: `tests/schema/public_type_snapshots.rs`,
`tests/schema/schema_snapshots.rs`, `tests/guardrails/no_execution_scan.rs`.

## Where the docs live
Start at `crates/bijux-dna-stage-contract/docs/INDEX.md` and follow the crate docs listed above.

## Start here in code
`src/lib.rs`, then `src/execution_plan/mod.rs`, `src/stage_plan/mod.rs`, `src/plan_run/mod.rs`, and `src/executor_registry/mod.rs`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-stage-contract/docs/CHANGE_RULES.md`.
