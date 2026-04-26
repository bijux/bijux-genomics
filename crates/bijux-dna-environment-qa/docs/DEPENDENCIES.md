# bijux-dna-environment-qa Dependencies

This crate is allowed to be heavier than production crates because it is the QA edge. The graph must
still point one way: environment QA consumes lower contracts; production layers must not consume
environment QA.

## Allowed Production Dependencies

- `anyhow`: command and QA workflow error context.
- `bijux-dna-analyze`: QA record types and SQLite/JSONL persistence APIs.
- `bijux-dna-core`: execution contracts, stage IDs, and tool IDs.
- `bijux-dna-domain-fastq`: FASTQ execution-support roster and stage constants.
- `bijux-dna-environment`: platform, image, Docker tool, and smoke helper contracts.
- `bijux-dna-infra`: directory creation, atomic writes, fixture IO, and config helpers.
- `bijux-dna-runtime`: runtime manifest loading for QA stage/tool discovery.
- `clap`: binary argument parsing.
- `rusqlite`: QA SQLite record store.
- `serde_json`: QA summary and artifact fixture JSON.
- `sha2`: input and output digest calculation.
- `tracing`: warnings for image resolution and seqkit parsing drift.
- `uuid`: per-run container/output names.

## Allowed Test Dependencies

- `bijux-dna-policies`: workspace guardrails.
- `bijux-dna-testkit`: deterministic fixture and artifact-aware temp helpers.

## Forbidden Dependencies

This crate must not depend on API, CLI router, planner, pipeline, runner backend, stage execution,
benchmark, database-fetch, science, or domain crates other than the FASTQ domain contract it is
explicitly validating.

## Verification

Use:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment-qa --no-default-features --test boundaries
```
