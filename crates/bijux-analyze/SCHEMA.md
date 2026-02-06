# SCHEMA

Aggregate schema definitions live under `src/aggregate/schema/fields/` and are
split by concern:
- `core.rs` for shared identifiers and core metrics fields.
- `bench.rs` for benchmarking-specific fields.
- `stage_metrics.rs` for stage-specific metric fields.

Registry wiring remains in `schema/defs.rs` and `schema/lookup.rs`.
