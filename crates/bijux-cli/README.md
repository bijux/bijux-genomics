# bijux-cli

## What this crate does
User-facing CLI for planning, dry-run, execution, reporting, and audits.

## What it must not do (boundaries)
No direct runner/engine dependencies; API only.

## Role in the stack
Upstream: users. Downstream: API.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/COMMANDS.md`, `docs/CLI_CONVENTIONS.md`, `docs/DRY_RUN.md`, `docs/UX_ERRORS.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
Dry-run outputs and help snapshots.

## Effects & determinism guarantees
CLI effects limited to input/output and API invocation. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/dry_run_fastq_golden.rs`, `tests/docs_help_snapshots.rs`, `tests/no_process_spawn.rs`, `tests/architecture_guardrail.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
