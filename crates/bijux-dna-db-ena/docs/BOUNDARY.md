# Boundary

Owner: ENA data source integration.

`bijux-dna-db-ena` owns ENA metadata selection, filereport decoding, download
planning, and ENA file transfer helpers. It is a data-source adapter, not a
pipeline planner or stage runner.

## Owns

- Typed ENA query selectors for projects, samples, and explicit accessions.
- ENA filereport URL construction, required-field selection, header validation,
  row decoding, and sample filtering.
- ENA result/source selection records.
- Deterministic download task planning under caller-provided output roots.
- The `bijux-dna-db-ena` helper binary for direct query and download workflows.
- Manifest persistence for the helper binary.

## Does Not Own

- Pipeline planning, stage selection, or scientific workflow execution.
- Runtime backend orchestration beyond local ENA file transfer.
- Workspace path policy outside caller-provided output and manifest paths.
- Corpus command UX in the top-level `bijux-dna` CLI.
- Reference database handling or non-ENA data-source contracts.

## Allowed Effects

- Network reads from ENA filereport and file endpoints when callers request
  query or download operations.
- Filesystem writes only to explicit output directories and manifest paths
  supplied through `DownloadConfig` or CLI arguments.
- Directory creation under those caller-provided paths.

## Forbidden Dependencies

This crate must not depend on planner, engine, runner, runtime, stage,
environment, API, domain, benchmark, or product CLI crates. The top-level
`bijux-dna` CLI may depend on this crate, but this crate must not depend back on
the CLI.

Dev-only guardrail dependencies are allowed for policy tests.

## Validation

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ena --test boundaries --no-default-features
```
