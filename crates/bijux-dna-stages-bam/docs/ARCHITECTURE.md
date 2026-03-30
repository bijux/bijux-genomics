# Architecture

## Goals
- Keep the crate root thin and explicit.
- Separate plugin orchestration from metrics parsing.
- Group BAM metrics by enduring concern instead of one catch-all file.

## Source tree

```text
src/
├── lib.rs
├── metrics/
│   ├── alignment.rs
│   ├── contamination.rs
│   ├── coverage.rs
│   ├── damage.rs
│   ├── discovery.rs
│   ├── mod.rs
│   └── quality.rs
├── observer/
│   └── mod.rs
├── plugin/
│   ├── invocation.rs
│   ├── mod.rs
│   └── output/
│       ├── collected_metrics.rs
│       ├── envelope.rs
│       └── mod.rs
├── stage_specs/
│   └── mod.rs
└── surface.rs
```

## Responsibilities
- `surface.rs`: crate-level public API surface and implemented stage list.
- `metrics/`: BAM output parsing grouped by alignment, coverage, quality, damage, and contamination concerns.
- `observer/`: observer parser re-exports for contract-facing consumers.
- `plugin/`: stage-plugin handling, invocation materialization, and metrics-envelope construction.
- `stage_specs/`: BAM stage/domain re-exports for planner-facing callers.

## Change rules
- Add root files only for enduring top-level concerns.
- Prefer explicit submodules over adding more logic to `mod.rs`.
- Update this document and the boundary architecture test together when the tree changes intentionally.
