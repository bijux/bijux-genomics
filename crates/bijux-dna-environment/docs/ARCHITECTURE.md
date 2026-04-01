# Architecture

This file is a pointer map. The environment contract is intentionally documented in small focused docs instead of one long architecture essay.

## Layout
- `public_api/` exposes the stable public crate surface through `stable_surface.rs`.
- `build/` owns build-time image defaults, models, version parsing, and dedicated build entrypoints.
- `resolve/entrypoints.rs` exposes the curated resolver functions while `resolve/facade.rs` owns the resolver struct facade.
- `resolve/cache/` owns cache roots and image path derivation.
- `resolve/catalog/` owns catalog assembly, raw TOML loading, registry digest hydration, and image reference synthesis.
- `resolve/reference/` owns reference registration, digest hashing, and index preparation.
- `resolve/types/` owns resolver models and parse-only records.
- `resolve/` owns runtime platform selection, shell and smoke helpers, plus the stable resolve export surface.
- `runtime_spec/` owns the pure runtime pairing between platform and runner.
- `lib.rs` stays thin and re-exports the supported surface.

## Change rules
- Add new root files only for enduring crate-level concerns.
- Prefer focused `build/` and `resolve/` modules over catch-all expansion.
- Keep root module exports in dedicated `stable_surface.rs` files instead of root modules.
- Update this map and the architecture boundary contract together when the tree changes intentionally.

## Pointers
- `INDEX.md` for the document map.
- `ENV_REFERENCE.md` and `ENV_MATRIX.md` for runtime behavior.
- `CACHE_SEMANTICS.md`, `BOUNDARY.md`, and `CHANGE_RULES.md` for extension rules.
