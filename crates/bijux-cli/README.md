# bijux-cli

## What this crate does
User-facing CLI for planning, dry-run, execution, reporting, and audits.

## What it must not do (boundaries)
No direct engine or runner calls; the CLI depends on `bijux-api` only.

## What the CLI guarantees
- Deterministic output for identical inputs.
- Dry-run output stability (manifest + graph shape).
- No hidden side effects beyond writing declared output artifacts.

## Effects & determinism guarantees
CLI output is deterministic for the same inputs; effects are limited to requested output files.

## Public API / entrypoints
CLI subcommands documented in `docs/COMMANDS.md`.

## Key contracts it owns/consumes
Owns CLI UX and output-format contracts; consumes API responses and schemas from `bijux-api`.

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
See `docs/TESTS.md`. Key tests: `tests/contracts/cli_contracts.rs`, `tests/contracts/dry_run/fastq_golden.rs`,
`tests/contracts/help/cli_help.rs`, `tests/snapshots/`.

## Where the docs live
Start at `docs/INDEX.md` and follow the command and output docs above.
