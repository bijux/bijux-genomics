# Architecture

This file is a pointer map. The environment contract is intentionally documented in small focused docs instead of one long architecture essay.

## Layout
- `surface.rs` exposes the public crate surface.
- `build/` owns build-time image defaults, models, and version parsing.
- `resolve/` owns runtime catalog loading, platform selection, cache paths, smoke commands, and reference preparation.
- `runtime_spec.rs` owns the pure runtime pairing between platform and runner.
- `lib.rs` stays thin and re-exports the supported surface.

## Change rules
- Add new root files only for enduring crate-level concerns.
- Prefer focused `build/` and `resolve/` modules over catch-all expansion.
- Update this map and the architecture boundary contract together when the tree changes intentionally.

## Pointers
- `INDEX.md` for the document map.
- `ENV_REFERENCE.md` and `ENV_MATRIX.md` for runtime behavior.
- `CACHE_SEMANTICS.md`, `BOUNDARY.md`, and `CHANGE_RULES.md` for extension rules.
