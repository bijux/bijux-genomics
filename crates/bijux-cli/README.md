# bijux-cli

## What this crate does
User-facing command-line interface for planning, dry-running, executing, and auditing Bijux pipelines.

## What it must not do (boundaries)
Must not execute tools directly or reach into runner/engine internals. It calls the API only.

## Public API / entrypoints
CLI commands documented in `docs/COMMANDS.md` and conventions in `docs/CLI_CONVENTIONS.md`.

## Key contracts it owns/consumes
Consumes API responses and renders deterministic output. See `docs/DRY_RUN.md` and `docs/UX_ERRORS.md`.

## Effects & determinism guarantees
Only effects are reading inputs and invoking API calls. Help output and dry-run artifacts are snapshot-tested.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/dry_run_fastq_golden.rs`, `tests/docs_help_snapshots.rs`, `tests/no_process_spawn.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/COMMANDS.md`, `docs/DRY_RUN.md`, and `docs/UX_ERRORS.md`.

## Artifacts / Contracts
Dry-run emits a manifest and plan summary; see `tests/snapshots/preprocess_artifacts_tree.txt`.

## Failure modes
CLI errors map API failures into actionable messages; see `docs/UX_ERRORS.md`.

## Stability
Help output and schemas are stable; snapshot updates follow `docs/CHANGE_RULES.md`.
