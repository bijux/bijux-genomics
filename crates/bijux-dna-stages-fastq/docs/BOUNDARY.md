# Boundary

`bijux-dna-stages-fastq` owns FASTQ stage contracts, observer parsers, metrics
normalization, runtime-interpretation classification, and stage-plugin output
envelopes for already-planned stages.

## Allowed Inputs

- FASTQ domain stage contracts and execution-support truth from
  `bijux-dna-domain-fastq`.
- Shared stage contracts from `bijux-dna-stage-contract`.
- Existing stage input and output files referenced by a `StagePlanV1`,
  `ArtifactRef`, or fixture.
- Fixture snapshots under `tests/fixtures/**` and `tests/snapshots/**`.

## Allowed Effects

- Read existing FASTQ, JSON, TSV, and text outputs referenced by a plan or
  fixture.
- Write crate-owned observer artifacts through explicit `observer::artifacts`
  helper functions.
- Build deterministic invocation, report, warning, event, and metrics envelope
  values in memory.

## Forbidden Effects

- No process spawning, shell execution, container invocation, network access, or
  environment setup.
- No planner orchestration, pipeline composition, engine scheduling, runner
  ownership, or CLI/API command surfaces.
- No tool selection or command-template construction.

## Dependency Boundary

Allowed normal dependencies are core contracts, FASTQ domain contracts, shared
stage contracts, infra helpers for hashing and governed JSON writes, serde,
serde_json, anyhow, flate2, and tracing. Forbidden normal dependencies include
planner crates, runner crates, engine crates, API/CLI crates, pipeline crates,
environment crates, and benchmark crates.

## Validation

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-fastq --test boundaries --no-default-features
```
