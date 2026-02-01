# bijux-analyze Architecture Map

## Ownership map
- Metrics live in `crates/bijux-analyze/src/aggregate/metrics/` (schemas + invariants).
- Metric registry tables live in `crates/bijux-analyze/src/aggregate/registry/`.
- SQL queries live in `crates/bijux-analyze/src/load/sqlite/`.
- Report builders live in `crates/bijux-analyze/src/report/`.
- Orchestration lives only in `crates/bijux-analyze/src/pipeline/`.

## Layer boundaries
- `load/` does IO + schema validation only.
- `model/` is typed IR (no IO).
- `aggregate/` computes rollups and stats.
- `decision/` compares, scores, explains.
- `report/` builds report models + renderers.
- `pipeline/` composes the steps; only place that can cross layers.

## Module depth rule
Allowed: `src/a/b/c.rs` plus `mod.rs` at each level.
Disallowed: deeper than 3 levels (except `mod.rs`).
