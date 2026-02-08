# Boundaries

## What bijux-dna-core is
- The single source of truth for serialized contract types, IDs, and canonicalization.
- Pure data definitions + validation logic for contracts.

## What bijux-dna-core must not contain
- Tool selection logic.
- Command assembly or execution plans beyond pure contract construction.
- Filesystem effects beyond pure serialization helpers.
- Runtime scheduling, execution, or side-effecting IO.

## Allowed effects
- Deterministic serialization/deserialization.
- Pure validation (no network, no filesystem mutation).
- Hashing and canonicalization of in-memory data.

## OWNERSHIP
- IDs (PipelineId/StageId/ToolId/MetricId): bijux-dna-core.
- Defaults/profiles: bijux-dna-pipelines.
- Param schemas: domain crates.
- Metric semantics: domain crates.
- Artifact layout: bijux-dna-runtime.
- Report schema/rendering: bijux-dna-analyze.
