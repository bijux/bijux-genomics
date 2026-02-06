# bijux-core Public API (Stable)

This crate is the contract bible for Bijux. Only the modules and types listed
below are considered stable public API. Anything else is internal and may
change without notice.

## Public Modules
- `contract`
- `execution`
- `ids`
- `metadata`
- `metrics`
- `metrics_registry`
- `primitives`
- `run_index`
- `selection`

## Core Contract Types
- `contract::ContractVersion`
- `execution::ExecutionGraph`, `execution::ExecutionStep`, `execution::ExecutionEdge`
- `execution::PlanPolicy`, `execution::RetryPolicy`
- `metrics::ToolInvocationV1`, `metrics::MetricsEnvelope`
- `contract::StageSpec`, `contract::StageIO`, `contract::ArtifactSpec`
- `primitives::CacheKey`, `primitives::CommandSpecV1`, `primitives::ContainerImageRefV1`
- `ids::*` (RunId, StepId, StageId, ToolId, ArtifactId, ProfileId, PipelineId)
- `primitives::BijuxError`, `primitives::Result`

If you need a new public type, update this file and add a compatibility note in
`docs/contract_compatibility.md`.
