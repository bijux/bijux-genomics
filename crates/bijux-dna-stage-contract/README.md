# bijux-dna-stage-contract

## What this crate does

`bijux-dna-stage-contract` defines the shared planning contract between
planners, stage crates, policy checks, runtime, and runner code. It owns stage
plans, execution plans, artifact-bound plan edges, stage plugin payloads, and
executor-readiness metadata.

## What this crate is NOT

- Not a runtime crate.
- Not a runner crate.
- Not a Docker, Apptainer, or environment crate.
- Not a CLI parser or command router.
- Not a planner selection-policy crate.

## Role in the stack

Upstream dependency: `bijux-dna-core` for typed IDs, artifact contracts,
metrics envelopes, command specs, and canonical JSON helpers.

Downstream consumers: planners, stage crates, engine/runtime/runner handoff
code, API surfaces, and policy checks.

## Public API / entrypoints

Start with `docs/INDEX.md`, then use:

- `docs/BOUNDARY.md` for ownership and dependency rules.
- `docs/COMMANDS.md` for the callable operation SSOT.
- `docs/CONTRACT.md` for versioning and serialized-plan terminology.
- `docs/PUBLIC_API.md` for exported modules, types, schema examples, and
  snapshots.

## Public types

- `ExecutionPlan`, `PlanEdge`, and `PlanValidationContext`.
- `StagePlanV1`, `StagePlanJsonV1`, `PlanDecisionReason`, and
  `PlannedArtifactV1`.
- `RunExecutionPlan` and `PlannerContractV1`.
- `StageInvocationV1`, `StagePluginOutputV1`, `StageReportPartV1`, and
  `StageEventHintV1`.
- `StageExecutorEntry`, `ReadinessBadge`, and `StageDomain`.

## Key contracts it owns/consumes

- Owns stage-plan, execution-plan, stage plugin, and executor registry contract
  shapes.
- Consumes core IDs, artifact references, `StageIO`, `ToolConstraints`,
  `CommandSpecV1`, `ContainerImageRefV1`, and metrics envelopes.
- Keeps public fixture snapshots under `tests/fixtures/`.

## Effects & determinism guarantees

This crate is pure contract code. It may serialize, validate, and hash contract
payloads, but it must not spawn processes, write runtime artifacts, mutate
source trees, or access the network. See `docs/EFFECTS.md`.

## How to run its tests

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --no-default-features
```

See `docs/TESTS.md` for focused suite commands and test ownership.

## Documentation Layout

The only root Markdown file is this README. All other documentation lives under
`docs/`, capped at 10 files:

- `ARCHITECTURE.md`
- `BOUNDARY.md`
- `CHANGE_RULES.md`
- `COMMANDS.md`
- `CONTRACT.md`
- `EFFECTS.md`
- `INDEX.md`
- `PUBLIC_API.md`
- `TESTS.md`

`docs/EXAMPLE_PLAN.json` is the single governed non-Markdown example fixture in
the docs allowance.

## Start here in code

`src/lib.rs`, then:

- `src/stage_plan/mod.rs`
- `src/execution_plan/mod.rs`
- `src/plan_run/mod.rs`
- `src/executor_registry/mod.rs`
- `src/stage_plugin.rs`

## Failure modes

Primary failures surface as boundary, contract, schema, or determinism
violations. Inspect `docs/TESTS.md` for the responsible suite.

## Stability

Contract and behavior changes follow `docs/CHANGE_RULES.md`.
