# bijux-dna-runtime Docs Index

This directory is the single documentation home for `bijux-dna-runtime`. The crate root keeps only `README.md`.

## Core Contracts
- `RUNTIME_CONTRACT.md` defines the runtime contract, terminology, compatibility, telemetry schema, and change rules.
- `ARTIFACTS.md` lists runtime-owned run-layout and tool-run artifacts.
- `PUBLIC_API.md` lists stable modules and root exports.
- `BOUNDARY.md` defines allowed responsibilities and forbidden ownership.

## Operational Boundaries
- `COMMANDS.md` is the single source of truth for runtime command ownership.
- `DEPENDENCIES.md` documents the allowed dependency graph.
- `EFFECTS.md` documents allowed filesystem, time, telemetry, and forbidden process/network effects.

## Maintenance
- `ARCHITECTURE.md` maps source layout to ownership.
- `TESTS.md` maps tests to contracts and failure meaning.

## Change Rules
- Runtime schema changes must be versioned and reflected in fixtures.
- Contract changes must update docs and tests together.
- Dependency changes must update `DEPENDENCIES.md` and dependency boundary coverage.
- Command ownership changes must update `COMMANDS.md` and command inventory coverage.
