# bijux-dna-science

## What this crate does
Compiles authored science specs into deterministic traceability outputs and science release bundles.

## What it must not do
No workflow execution, no stage orchestration, and no direct tool launching.

## First implemented slice
FASTQ environment and container support:

- admitted stage-tool surface
- governed defaults
- planned tools kept outside the closed runtime surface
- environment and container references backing those decisions

## Public entrypoints
- library: `app::validate_workspace`, `app::build_workspace`, `app::trace_workspace`,
  `app::release_workspace`
- binary: `cargo run -p bijux-dna-science -- <command>`

## Where the docs live
See `docs/architecture.md`, `docs/cli.md`, `docs/schema-model.md`, and `docs/release-model.md`.
