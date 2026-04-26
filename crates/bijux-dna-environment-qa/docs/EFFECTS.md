# bijux-dna-environment-qa Effects

This crate is intentionally effectful. The boundary is not "no effects"; it is "effects only in the
QA crate and only under explicit command surfaces."

## Allowed Reads

- Runtime platform and image catalog inputs from `bijux-dna-environment`.
- FASTQ QA datasets and fixture artifacts.
- Existing QA JSONL/SQLite records under `artifacts/image-qa/<platform>/`.
- Local Docker/Apptainer image state.
- Host environment variables used by command options and environment helpers.

## Allowed Writes

- QA records, summaries, SQLite databases, logs, and generated subsets under
  `artifacts/image-qa/<platform>/`.
- Per-run QA output directories under the repository `artifacts/` tree.
- Docker image builds when `build_docker_images` is explicitly invoked.

## Allowed Processes

The full process inventory lives in `COMMANDS.md`. Fast tests must not execute those commands
against the host; they use fake runners or fixture data.

## Offline Policy

- Default tests are offline.
- Docker, Apptainer, network pulls, and long-running image QA require explicit operator invocation.
- External datasets are never fetched by default tests.

## Forbidden Effects

- Runtime QA must not mutate source, docs, configs, or checked-in fixtures.
- Production crates must not import this crate to gain command execution.
- Network access must not happen in default tests.
- QA output must not be written to `target/qa`, OS temp roots, or ad hoc source-tree directories.
