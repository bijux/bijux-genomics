# Effects

`bijux-dna-pipelines` is a pure contract crate.

## Allowed
- Deterministic profile construction.
- Deterministic serialization and hashing.
- Reading crate-owned docs, tests, and fixtures during tests.
- Writing Cargo build and test artifacts under `artifacts/` through normal test execution.

## Forbidden
- Process spawning.
- Network access.
- Runtime tool discovery.
- Product execution.
- Undeclared file writes.
- CLI argument parsing or command routing.

## Enforcement
- Repository policy guardrails run through `tests/boundaries.rs`.
- Command ownership is locked by `tests/boundaries/command_inventory.rs`.
- Source-tree shape is locked by `tests/boundaries/architecture_tree.rs`.
