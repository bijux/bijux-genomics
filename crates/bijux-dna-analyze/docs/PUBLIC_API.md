# Public API

## Crate Root
The crate root is intentionally small:

- `analyze_run`: the canonical analysis entrypoint
- `pub use public_api::*;`: the curated stable surface

`src/lib.rs` must not grow direct compatibility shims for every helper. New stable exports belong
in `src/public_api/` so the public surface remains reviewable.

## Stable Surface
`src/public_api/mod.rs` re-exports:

- request and response models from `src/api/`
- versioned contract helpers from `src/contracts/`
- report, load, aggregate, failure, and export helpers intended for reuse
- decision comparison and ranking helpers under explicit namespaces
- metric semantics helpers shared with downstream consumers

## Internal Modules
Some modules remain public for existing tests and downstream integration points. Treat those as
compatibility surfaces, not permission to bypass ownership boundaries:

- `aggregate` owns metric schemas and aggregation helpers.
- `decision` owns comparison, scoring, ranking, and traces.
- `exports` owns summary and dashboard artifact writers.
- `failure` owns failure classification and hints.
- `load` owns artifact loading and optional SQLite/parquet readers.
- `model` owns typed internal analysis records.
- `report` owns report construction and rendering.

## Change Rule
Adding or removing a stable item requires:

- updating this document when the intended surface changes
- updating `tests/snapshots/bijux-dna-analyze__schemas__public_api.txt`
- running the public API snapshot through `docs/COMMANDS.md`
