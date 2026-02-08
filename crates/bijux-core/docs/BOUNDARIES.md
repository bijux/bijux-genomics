# Boundaries

## What bijux-core is
- The single source of truth for serialized contract types, IDs, and canonicalization.
- Pure data definitions + validation logic for contracts.

## What bijux-core must not contain
- Tool selection logic.
- Command assembly or execution plans beyond pure contract construction.
- Filesystem effects beyond pure serialization helpers.
- Runtime scheduling, execution, or side-effecting IO.

## Allowed effects
- Deterministic serialization/deserialization.
- Pure validation (no network, no filesystem mutation).
- Hashing and canonicalization of in-memory data.

## OWNERSHIP
- IDs (PipelineId/StageId/ToolId/MetricId): bijux-core.
- Defaults/profiles: bijux-pipelines.
- Param schemas: domain crates.
- Metric semantics: domain crates.
- Artifact layout: bijux-runtime.
- Report schema/rendering: bijux-analyze.
