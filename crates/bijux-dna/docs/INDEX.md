# bijux-dna Documentation Index

## Scope
`bijux-dna` is the CLI adapter crate. It owns command parsing, command routing, CLI rendering,
operator errors, and the process boundary. It does not own scientific domain semantics or backend
execution.

## Start Here
- [BOUNDARY.md](BOUNDARY.md) defines the crate contract.
- [ARCHITECTURE.md](ARCHITECTURE.md) defines the source and test tree.
- [COMMANDS.md](COMMANDS.md) is the single source of truth for commands managed by this crate.

## Contract Docs
- [PUBLIC_API.md](PUBLIC_API.md) defines the exported Rust surface.
- [DRY_RUN.md](DRY_RUN.md) defines planning-only behavior.
- [EFFECTS.md](EFFECTS.md) defines allowed reads, writes, process execution, and network behavior.
- [OUTPUT_FORMATS.md](OUTPUT_FORMATS.md) defines terminal, JSON, report, and error output rules.
- [CHANGE_RULES.md](CHANGE_RULES.md) defines breaking-change review rules.
- [TESTS.md](TESTS.md) maps each contract surface to tests.

## Operator Support
Minimal bug reports should include the command and flags, relevant output artifacts, the
`run_manifest.json` or dry-run plan when available, and the terminal or JSON error payload. Error
categories are governed by [EFFECTS.md](EFFECTS.md) and [OUTPUT_FORMATS.md](OUTPUT_FORMATS.md).

## Allowed Documentation Files
This crate keeps one root `README.md` and these ten files under `docs/`:

- `ARCHITECTURE.md`
- `BOUNDARY.md`
- `CHANGE_RULES.md`
- `COMMANDS.md`
- `DRY_RUN.md`
- `EFFECTS.md`
- `INDEX.md`
- `OUTPUT_FORMATS.md`
- `PUBLIC_API.md`
- `TESTS.md`

No README or markdown placeholder files belong under `src/` or `tests/`.
