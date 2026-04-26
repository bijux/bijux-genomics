# bijux-dna-dev Docs Index

## Scope
- [SCOPE.md](SCOPE.md)

## Effects
- Repository-scoped filesystem reads and writes for automation artifacts.
- Process execution for explicit development workflows and repository checks.

## Boundaries
- [ARCHITECTURE.md](ARCHITECTURE.md)
- [BOUNDARY.md](BOUNDARY.md)
- [PUBLIC_API.md](PUBLIC_API.md)
- [TESTS.md](TESTS.md)

## Command Catalog
- [COMMANDS.md](COMMANDS.md) is the single source of truth for the command groups managed by this crate.

## Extension Points
- Add new cataloged commands under `src/catalog` and wire them through `src/commands`.
- Extend runtime adapters under `src/runtime` when a workflow needs a new boundary-owned capability.

## How to Test
- Run `cargo test -p bijux-dna-dev`.
- Read [TESTS.md](TESTS.md) for command-module coverage.
- Run `cargo test -p bijux-dna-policies policy__boundaries__docs_index_quality__docs_index_has_required_sections -- --nocapture`.
