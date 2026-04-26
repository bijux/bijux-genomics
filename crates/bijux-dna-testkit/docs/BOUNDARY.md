# Boundary

`bijux-dna-testkit` owns reusable test helpers. It is consumed by tests across
the workspace and must not become part of product runtime behavior.

## Owned Here

- Deterministic clocks, seeded RNG helpers, timestamp-field stripping, and stable
  ordering assertions.
- Fixture text and JSON readers.
- JSON shape assertions for tests.
- Snapshot text and JSON normalization.
- Snapshot naming and deterministic test environment setup.
- Temporary test directories, contained relative test paths, and sorted directory
  listings.
- Workspace-root and policy-text helpers for tests.

## Owned Elsewhere

- Domain semantics belong in domain and stage crates.
- Production runtime behavior belongs in runtime and runner crates.
- CLI parsing and user command routing belong in command/API crates.
- Planner policy belongs in planner crates.
- Environment provisioning belongs in environment crates.

## Boundary Risks

- Adding required dependencies on product crates makes this crate a production
  dependency owner.
- Adding process, network, or source mutation effects makes test behavior less
  reproducible.
- Adding broad helper buckets hides ownership and makes public API drift harder
  to review.
