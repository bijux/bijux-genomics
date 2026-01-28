# BAM Readiness Checklist

This checklist exists to prevent copying FASTQ logic into BAM work.

## Reuse (required)
- Run layout and metadata schemas (RunMetadataV1, StageMetadataV1, ToolExecutionMetadataV1)
- Input assessment flow and canonical sample naming
- Engine run/analyze separation and run index
- Event stream schema (RunEvent)
- Metrics collection via bijux-measure
- Report/selection in bijux-analyze

## Do not duplicate
- Tool selection logic inside domains
- Benchmark gates inside engine
- Domain-specific execution in engine services

## Must define before BAM execution
- BAM artifact types and invariants
- Stage contracts and compatibility
- Minimal pipeline and canonical pipeline
- Metrics schema + delta semantics

## Exit criteria
- BAM runs produce identical run layout
- Domain has no execution side effects
- Engine imports no BAM modules
