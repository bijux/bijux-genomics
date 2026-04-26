# bijux-dna Docs Index

## Scope
`bijux-dna` is the CLI adapter crate. It owns command parsing, command routing, CLI rendering,
operator errors, and the process boundary. It does not own scientific domain semantics or backend
execution.

## Effects
Effects are constrained by [EFFECTS.md](EFFECTS.md). The CLI may read repository configuration and
write explicitly requested command outputs; it must not spawn tools or perform hidden network work.

## Boundaries
- [BOUNDARY.md](BOUNDARY.md) defines the crate contract.
- [PUBLIC_API.md](PUBLIC_API.md) defines the exported Rust surface.
- [ARCHITECTURE.md](ARCHITECTURE.md) defines the source and test tree.

## Extension Points
Use [CHANGE_RULES.md](CHANGE_RULES.md) before changing command names, output formats, public API, or
snapshot-locked help text.

## Commands
[COMMANDS.md](COMMANDS.md) is the single source of truth for commands managed by this crate.

## Output Formats
[OUTPUT_FORMATS.md](OUTPUT_FORMATS.md) covers terminal text, JSON, dry-run artifacts, reports, and
operator errors.

## How to Test
[TESTS.md](TESTS.md) maps each contract surface to the tests that protect it.

## Allowed Docs
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
