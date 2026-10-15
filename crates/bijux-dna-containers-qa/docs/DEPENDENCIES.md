# bijux-dna-containers-qa Dependencies

This crate is allowed to be heavier than production crates because it is the QA edge. The graph must
still point one way: environment QA consumes lower contracts, and only the runner consumes its
stable qualification facade.

## Allowed Production Dependencies

- `anyhow`: command and QA workflow error context.
- `bijux-dna-analyze`: QA record types and SQLite/JSONL persistence APIs.
- `bijux-dna-core`: execution contracts, stage IDs, and tool IDs.
- `bijux-dna-domain-fastq`: FASTQ execution-support roster and stage constants.
- `bijux-dna-containers`: platform, image, Docker tool, and smoke helper contracts.
- `bijux-dna-infra`: directory creation, atomic writes, fixture IO, and config helpers.
- `bijux-dna-runtime`: runtime manifest loading for QA stage/tool discovery.
- `bijux-dna-stage-contract`: canonical stage/tool bindings and pure command projection.
- `schemars`: generated qualification specification source schema.
- `clap`: binary argument parsing.
- `regex`: version-contract evaluation for qualification probe observations.
- `rusqlite`: QA SQLite record store.
- `serde`: generic immutable QA evidence serialization.
- `serde_json`: QA summary and artifact fixture JSON.
- `serde_yaml`: declarative qualification source loading.
- `sha2`: input and output digest calculation.
- `tracing`: warnings for image resolution and seqkit parsing drift.
- `uuid`: per-run container/output names.

## Allowed Test Dependencies

- `bijux-dna-process`: bounded isolated QA contract fixture processes.

- `bijux-dna-policies`: workspace guardrails.
- `bijux-dna-testkit`: deterministic fixture and artifact-aware temp helpers.

## Forbidden Dependencies

This crate must not depend on API, CLI router, planner, pipeline, runner backend, stage execution,
benchmark, database-fetch, science, or domain crates other than the FASTQ domain contract it is
explicitly validating.

## Verification

Use:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers-qa --no-default-features --test boundaries
```

## Qualification compilation boundary

The default `verification-tools` feature provides the existing image, runtime and security verification modules and five command binaries. It enables their command parsing, SQLite analysis, FASTQ behavior, logging and supervised-process dependencies. With default features disabled, qualification source admission, compilation, planning, materialization and verdict evaluation remain available without those QA harness dependencies.

The workspace runner consumes this qualification surface without enabling `verification-tools`. The public API facade enables it explicitly because it exports the verification services. Features are additive: a joint build containing the API can still enable the broader surface; package-scoped runner builds are the independent compilation boundary.
