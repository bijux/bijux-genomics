# bijux-dna-planner-bam Boundary Contract

Owner: BAM planner
Scope: deterministic BAM stage planning, tool selection, execution-graph assembly, and explain payload contracts.

## Allowed Inputs
- BAM domain stage contracts and typed IDs.
- BAM pipeline profiles from `bijux-dna-pipelines`.
- Stage plan contracts from `bijux-dna-stage-contract`.
- BAM stage adapter contract data from `bijux-dna-stages-bam`.
- Repository configuration used to choose deterministic tool candidates.

## Forbidden Dependencies
- Runner backends.
- CLI adapters and command routers.
- Engine execution.
- Environment probes and runtime discovery.
- Database, science orchestration, and analysis application crates.

## Forbidden Effects
- Process spawning.
- Network access.
- Product execution.
- Parsing tool output.
- Mutating generated configuration.

## Allowed Effects
- Pure deterministic plan construction.
- Reading repository-owned tool registry configuration.
- Fixture-backed tests and snapshot comparisons.

## Validation
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-planner-bam --no-default-features
```
