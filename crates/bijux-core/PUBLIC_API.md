# bijux-core Public API (Stable)

This crate is the contract bible for Bijux. Only the modules and types listed
below are considered stable public API. Anything else is internal and may
change without notice.

## Public Modules
- `contract`
- `ids`
- `metrics`
- `foundation`
- `prelude`

## Core Contract Types
- `contract::ContractVersion`
- `contract::ExecutionGraph`, `contract::ExecutionStep`, `contract::ExecutionEdge`
- `contract::PlanPolicy`, `contract::RetryPolicy`
- `metrics::ToolInvocationV1`, `metrics::MetricsEnvelope`
- `contract::StageSpec`, `contract::StageIO`, `contract::ArtifactSpec`
- `foundation::CacheKey`, `foundation::CommandSpecV1`, `foundation::ContainerImageRefV1`
- `ids::*` (RunId, StepId, StageId, ToolId, ArtifactId, ProfileId, PipelineId)
- `foundation::BijuxError`, `foundation::Result`

If you need a new public type, update this file and add a compatibility note in
`docs/contract_compatibility.md`.
