# bijux-cli

## What this crate does
User-facing CLI for planning, dry-run, execution, reporting, and audits.

## What the CLI guarantees
- Deterministic output for identical inputs.
- Dry-run output stability (manifest + graph shape).
- No hidden side effects beyond writing declared output artifacts.

## Boundaries
CLI depends on `bijux-api` only; it does not call engine or runner directly.

## Commands reference
`docs/COMMANDS.md` is the single authoritative command reference. README only summarizes.

## Output formats
See `docs/OUTPUT_FORMATS.md` for JSON/text expectations and snapshot links.

## Docs entrypoints
See `docs/INDEX.md`, `docs/COMMANDS.md`, `docs/CLI_CONVENTIONS.md`, `docs/DRY_RUN.md`,
`docs/OUTPUT_FORMATS.md`, `docs/UX_ERRORS.md`, `docs/CHANGE_RULES.md`.
