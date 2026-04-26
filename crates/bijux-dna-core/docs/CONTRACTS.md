# Contracts

`bijux-dna-core` owns serialized contracts and pure validation helpers that
downstream crates consume without redefining contract shape.

## Execution Contracts

`contract::execution` owns:

- `ExecutionGraph`, `ExecutionStep`, `ExecutionEdge`, and graph validation.
- `ExecutionContract` and `validate_execution_outputs`.
- `ExecutionManifest` and stage execution records.
- `PlanPolicy`, `RetryPolicy`, `ToolConstraints`, `StageIO`, `ArtifactSpec`,
  `ArtifactRole`, and path/port records.

Execution graphs are declarative. They may contain command templates and
container image refs as data, but they must not contain runtime execution state,
stage planner internals, or runner-specific behavior.

## Run Contracts

`contract::run` owns:

- `RunRecordV1`, `RunSpec`, and run index line/query helpers.
- `RunMetadataV1`, `StageMetadataV1`, `ToolExecutionMetadataV1`, and
  `ToolInvocationMetadataV1`.
- `ScientificProvenanceV1` and `ToolProvenanceV1`.
- Pipeline domain/spec records used by planners and APIs.

Run index helpers can read and query typed index files, but this crate does not
own runtime publish workflows or report persistence.

## Tooling Contracts

`contract::tooling` owns:

- `StageSpec`, `ToolManifest`, `ToolRegistry`, `ToolExecutionSpecV1`, and
  `ToolInvocationV1`.
- `Objective`, `ObjectiveSpec`, `ObjectiveWeights`, `ToolScore`,
  `Disqualification`, `StageSelection`, and `select_stage`.

Selection is pure scoring over typed candidate data. It does not execute tools
or choose runtime backends.

## Identifier Contracts

`ids` owns typed wrappers and parsing for pipeline, stage, tool, artifact, run,
step, and profile ids. `id_catalog` owns canonical constants for shared ids.
New shared id families belong here only when they are cross-crate contracts.

## Metrics Contracts

`metrics` owns metric ids, derived metric parsing, metrics schema ids, stage
metrics records, registry constants, and tool invocation metric payloads.
Domain-specific metric interpretation can live downstream, but shared metric
shape and id validation live here.

## Foundation Contracts

`foundation` owns shared model helpers exported through the prelude:

- command specs and container image refs
- cache and reproducibility identity records
- canonicalization and hashing helpers
- error and invariant records
- FASTQ input assessment records and helpers

Foundation helpers must stay generic and must not import orchestration,
planning, runtime, or product API behavior.

## Verification

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-core --test contracts --no-default-features
```
