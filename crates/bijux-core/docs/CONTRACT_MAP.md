# Contract Map

Single source of truth for core contract locations.

## Core contract families
- Execution planning: `ExecutionGraph`, `ExecutionStep`, `ExecutionEdge`, `PlanPolicy`
- Execution manifests and records: `ExecutionManifest`, `RunRecordV1`, `StageExecutionRecordV1`
- Run metadata and provenance: `RunMetadataV1`, `StageMetadataV1`, `ToolExecutionMetadataV1`, `ScientificProvenanceV1`, `ToolProvenanceV1`
- Tooling registry types: `StageSpec`, `ToolManifest`, `ToolRegistry`, `ToolExecutionSpecV1`
- Metrics registry and schemas: `metrics` module
- Identifiers: `ids` module

## Where they live
- `src/contract/execution/*` for execution graph and record contracts.
- `src/contract/run/*` for run metadata, provenance, and indices.
- `src/contract/tooling/*` for tooling registry and selection contract types.
- `src/metrics/*` for metrics schemas and registries.
- `src/ids.rs` for ID types.

## Notes
- Canonicalization and hashing helpers live under `src/contract/canonical.rs` and `src/foundation/*`.
- External crates must consume these contracts, not redefine them.
