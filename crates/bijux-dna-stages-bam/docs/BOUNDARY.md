# Boundary

## Role
`bijux-dna-stages-bam` owns BAM stage descriptions, BAM stage plugin
materialization, BAM observer parsers, and BAM metrics collection from declared
stage outputs.

The crate sits between BAM domain contracts and planner/runtime consumers. It
does not choose a pipeline, select tools for a user request, execute tools, or
own CLI/API command surfaces.

## Allowed Inputs
- BAM domain stage IDs and metric contracts from `bijux-dna-domain-bam`.
- Shared stage plugin contracts from `bijux-dna-stage-contract`.
- Existing files named by a `StagePlanV1` or by test fixtures when collecting
  output metrics and stable input fingerprints.
- Fixture snapshots under `tests/fixtures/**`.

## Allowed Effects
- Read existing fixture, input, or output files referenced by a test or stage
  plan.
- Build deterministic invocation envelopes and metrics envelopes.
- Parse observer output formats into stable BAM metric structures.

## Forbidden Effects
- No process spawning, shell execution, container invocation, network access, or
  config generation.
- No planner orchestration, runtime scheduling, or runner ownership.
- No filesystem writes in production code.

## Dependency Boundary
Allowed normal dependencies:

- `anyhow` for fallible plugin and parser contracts.
- `bijux-dna-core` for shared contract and metrics envelope types.
- `bijux-dna-domain-bam` for BAM stage IDs and BAM metrics.
- `bijux-dna-infra` only for stable file hashing used in input fingerprints.
- `bijux-dna-stage-contract` for shared stage plugin contracts.
- `serde` and `serde_json` for stable contract serialization.

Forbidden normal dependencies include CLI adapters, API orchestration crates,
engine crates, planner crates, runner crates, runtime crates, and environment
management crates.

## Validation
Run:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-bam --no-default-features
```
