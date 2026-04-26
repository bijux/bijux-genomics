# Architecture

`bijux-dna-testkit` is organized by test-support concern. The crate root
re-exports the stable helper surface; internal modules remain small enough to
reason about directly.

## Source Layout

- `src/lib.rs` exposes public modules and curated root re-exports.
- `src/public_api/` mirrors the stable root surface for callers that prefer a
  namespace over root imports.
- `src/determinism/` owns fixed clocks, seeded RNG support, timestamp-field
  stripping, and deterministic assertions.
- `src/fixtures/` owns fixture readers and JSON contract assertions.
- `src/snapshots/` owns snapshot naming, environment setup, text sanitization,
  and JSON normalization.
- `src/temp/` owns temp directory allocation, path derivation, directory
  listings, and `TestPaths`.
- `src/workspace_support/` owns workspace-root and policy-text helpers.

## Data flow

1. Tests import stable helpers from the crate root or `public_api`.
2. Snapshot helpers normalize text and JSON through dedicated pipelines.
3. Fixture readers load caller-owned test data without applying domain logic.
4. Temp helpers allocate isolated test locations and derive contained paths.

## Layout Rules

- Keep one source file per stable support concern unless a module needs a
  deliberate submodule split.
- Keep public re-exports curated in `src/lib.rs` and `src/public_api/surface.rs`.
- Update `tests/boundaries/architecture_tree.rs` with intentional tree changes.
