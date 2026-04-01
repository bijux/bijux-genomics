# bijux-dna-analyze Public API

The crate root stays intentionally small:
- `analyze_run`: canonical analysis entrypoint
- `pub use public_api::*;`: curated stable surface

`src/public_api/mod.rs` re-exports the durable API:
- request and response models from `src/api/`
- versioned contract helpers from `src/contracts/`
- report, load, aggregate, failure, and export helpers that are intended for reuse
- decision comparison and ranking helpers under explicit `compare` and `ranking` namespaces
- metric semantics helpers shared with downstream consumers

The root `src/lib.rs` no longer exposes contract, export, compare, or ranking shims directly.
New stable items should be added to `src/public_api/mod.rs`, not spread across `src/lib.rs`.
