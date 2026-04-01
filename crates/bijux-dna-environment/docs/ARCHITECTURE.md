# Architecture

This file is a pointer map. The environment contract is intentionally documented in small focused docs instead of one long architecture essay.

## Layout
- `public_api/` exposes the stable public crate surface.
- `build/` owns build-time image defaults, models, and version parsing.
- `resolve/facade.rs` exposes the curated resolver entrypoints.
- `resolve/cache/` owns cache roots and image path derivation.
- `resolve/types/` owns resolver models and parse-only records.
- `resolve/` owns runtime catalog loading, platform selection, smoke commands, and reference preparation.
- `runtime_spec/` owns the pure runtime pairing between platform and runner.
- `lib.rs` stays thin and re-exports the supported surface.

## Change rules
- Add new root files only for enduring crate-level concerns.
- Prefer focused `build/` and `resolve/` modules over catch-all expansion.
- Update this map and the architecture boundary contract together when the tree changes intentionally.

## Pointers
- `INDEX.md` for the document map.
- `ENV_REFERENCE.md` and `ENV_MATRIX.md` for runtime behavior.
- `CACHE_SEMANTICS.md`, `BOUNDARY.md`, and `CHANGE_RULES.md` for extension rules.
