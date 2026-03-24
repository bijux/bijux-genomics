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
CLI subcommands documented in `crates/bijux-dna/docs/COMMANDS.md`.

## Key contracts it owns/consumes
Owns CLI UX and output-format contracts; consumes API responses and schemas from `bijux-dna-api`.

## Artifacts / Contracts
See `crates/bijux-dna/docs/OUTPUT_FORMATS.md` and snapshots under `tests/snapshots/`.

## Failure modes
Most failures surface as contract snapshot diffs or API boundary violations.

## Commands reference
`crates/bijux-dna/docs/COMMANDS.md` is the single authoritative command reference. README only summarizes.

## Output formats
See `crates/bijux-dna/docs/OUTPUT_FORMATS.md` for JSON/text expectations and snapshot links.

## Docs entrypoints
See `crates/bijux-dna/docs/INDEX.md`, `crates/bijux-dna/docs/COMMANDS.md`, `crates/bijux-dna/docs/CLI_CONVENTIONS.md`, `crates/bijux-dna/docs/DRY_RUN.md`,
`crates/bijux-dna/docs/OUTPUT_FORMATS.md`, `crates/bijux-dna/docs/UX_ERRORS.md`, `crates/bijux-dna/docs/CHANGE_RULES.md`.

## How to run its tests
See `crates/bijux-dna/docs/TESTS.md`. Key tests: `tests/contracts/cli_contracts.rs`, `tests/contracts/dry_run/fastq_golden.rs`,
`tests/contracts/help/cli_help.rs`, `tests/snapshots/`.

## Where the docs live
Start at `crates/bijux-dna/docs/INDEX.md` and follow the command and output docs above.
