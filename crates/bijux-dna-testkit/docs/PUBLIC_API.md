# PUBLIC_API

This crate must stay deliberate and partitioned by support concern.

## Exported surface
- `determinism` — deterministic clocks, seeded RNG, timestamp-field stripping, and deterministic assertions.
- `fixtures` — fixture readers and JSON shape assertions.
- `public_api` — curated mirror of the stable root surface.
- `snapshots` — snapshot naming, environment setup, and normalization.
- `temp` — temp directory allocation and path helpers.
- `workspace_support` — workspace-root and policy-text helpers.

Stable root helpers are documented in the crate-level `PUBLIC_API.md`. Any new surface must update both docs and the public API snapshot.
