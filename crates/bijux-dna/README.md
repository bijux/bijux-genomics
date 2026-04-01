# bijux-dna

## What this crate does
User-facing CLI for planning, dry-run, execution, reporting, and audits.

## What it must not do (boundaries)
No direct engine or runner calls; the CLI depends on `bijux-dna-api` only.

## What the CLI guarantees
- Deterministic output for identical inputs.
- Dry-run output stability (manifest + graph shape).
- No hidden side effects beyond writing declared output artifacts.

## Effects & determinism guarantees
CLI output is deterministic for the same inputs; effects are limited to requested output files.

## Public API / entrypoints
Start with `PUBLIC_API.md`, `docs/ARCHITECTURE.md`, and `docs/COMMANDS.md`. The library surface is routed through `src/public_api/`, the crate-local launcher lives in `src/cli_entrypoint.rs`, and the process exit contract lives in `src/process_exit.rs`.

The command tree is intentionally partitioned: `src/commands/router/` owns routing, `src/commands/support/` owns shared command helpers, `src/commands/planning/` owns run planning, `src/commands/status/` owns status inspection, `src/commands/corpus/` owns curated corpus workflows, and `src/commands/fastq/meta/` owns FASTQ meta-command dispatch.

## Key contracts it owns/consumes
Owns CLI UX and output-format contracts; consumes API responses and schemas from `bijux-dna-api`.

## Artifacts / Contracts
See `docs/OUTPUT_FORMATS.md` and snapshots under `tests/snapshots/`.

## Failure modes
Most failures surface as contract snapshot diffs or API boundary violations.

## Commands reference
`docs/COMMANDS.md` is the single authoritative command reference. README only summarizes.

## Output formats
See `docs/OUTPUT_FORMATS.md` for JSON/text expectations and snapshot links.

## Docs entrypoints
See `docs/INDEX.md`, `docs/COMMANDS.md`, `docs/CLI_CONVENTIONS.md`, `docs/DRY_RUN.md`,
`docs/OUTPUT_FORMATS.md`, `docs/UX_ERRORS.md`, `docs/CHANGE_RULES.md`.

## How to run its tests
See `docs/TESTS.md`. Key coverage starts in `tests/boundaries.rs`, `tests/boundaries/architecture_tree.rs`, `tests/contracts/cli_behavior.rs`, `tests/contracts/dry_run/`, and `tests/snapshots/help/`.

## Where the docs live
Start at `docs/INDEX.md` and follow `docs/ARCHITECTURE.md`, `docs/COMMANDS.md`, `docs/OUTPUT_FORMATS.md`, and `docs/TESTS.md`.
