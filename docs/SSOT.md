# Single Source of Truth (SSOT)

- **IDs**: `bijux-core` owns typed IDs (`RunId`, `StageId`, `ToolId`, `PipelineId`).
- **Stage truth**: `bijux-stages-*` define stage specs and observers.
- **Tool truth**: planners own tool selection; stage contracts enumerate applicability.
- **Metrics definitions**: domain crates (`bijux-domain-fastq`, `bijux-domain-bam`).
- **Pipelines**: `bijux-pipelines` owns pipeline IDs and profiles.
