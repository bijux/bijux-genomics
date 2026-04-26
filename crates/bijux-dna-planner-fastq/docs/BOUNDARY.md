# Boundary

`bijux-dna-planner-fastq` owns deterministic FASTQ plan assembly. It selects tool bindings, composes stage plans, builds execution graphs, and exposes explainable planner metadata.

## Allowed Inputs
- FASTQ domain stage contracts and tool governance from `bijux-dna-domain-fastq`.
- Stage plan contract types from `bijux-dna-stage-contract`.
- FASTQ stage spec builders from `bijux-dna-stages-fastq`.
- Pipeline profiles from `bijux-dna-pipelines`.
- Repository-owned registry/configuration reads through `bijux-dna-infra`.

## Forbidden Inputs
- Runtime tool output parsing.
- Environment probing.
- CLI command routing.
- Runner, engine, API, database, analysis, or science orchestration state.

## Allowed Effects
- Pure plan construction.
- Deterministic serialization for snapshots and contract tests.
- Repository-owned config reads needed to resolve governed tool registry data.
- Planner diagnostics through `tracing`.

## Forbidden Effects
- Process spawning.
- Network access.
- Runtime tool discovery.
- Product execution.
- Generated configuration mutation.

## Validation
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-fastq --no-default-features
```
