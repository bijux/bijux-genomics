# Architecture

## Tree
- `src/public_api/` mirrors the curated stable surface.
- `src/determinism/` owns deterministic clocks, seeded RNG support, timestamp-field stripping, and deterministic assertions.
- `src/fixtures/` owns fixture readers and JSON contract assertions.
- `src/snapshots/` owns snapshot naming, environment setup, text sanitization, and JSON normalization.
- `src/temp/` owns temp directory allocation, path derivation, directory listings, and `TestPaths`.
- `src/workspace_support/` owns workspace-root and policy-text helpers.

## Data flow
1. Tests import stable helpers from the crate root or `public_api`.
2. Snapshot helpers normalize text and JSON through dedicated pipelines.
3. Temp helpers allocate isolated test locations and derive stable paths.
