# Contract Spine

## Core Contracts
- **ExecutionGraph**: The planner output used by the engine for orchestration.
- **StageSpec**: Stage contract describing inputs, outputs, and metrics roles.
- **RunManifest**: The run‑level truth artifact (graph hash, artifacts, toolchain versions).
- **ToolInvocation**: Tool identity + parameters + input/output fingerprints.
- **Metrics**: Per‑step metrics envelopes emitted by observers.
- **Reports**: Aggregated outputs produced by analyze and report aggregation.

## Flow
Planner → `ExecutionGraph` → Engine + Runner → Runtime recording → `RunManifest` + `ToolInvocation` + metrics → Analyze/Benchmark.

## Compatibility
Contract versions are explicit. Breaking changes require version bumps and policy updates.
