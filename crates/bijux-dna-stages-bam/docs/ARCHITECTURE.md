# Architecture

This document is intentionally short. It records the stable crate map and points to the docs that carry the detailed BAM stage contract.

## Layout
- `lib.rs` and `surface.rs` expose the public stage surface.
- `stage_specs/` owns planner-facing stage and domain re-exports.
- `plugin/` owns invocation materialization and output envelope construction.
- `observer/` owns observer-facing parser exports.
- `metrics/` owns BAM metrics grouped by enduring concerns such as alignment, coverage, quality, damage, and contamination.

## Change rules
- Add root files only for enduring crate-level concerns.
- Prefer explicit submodules over growing `mod.rs` files into catch-all hubs.
- Update this map and the boundary architecture test together when the layout changes intentionally.

## Pointers
- `INDEX.md` for the documentation map.
- `STAGE_CONTRACTS.md`, `STAGE_LIST.md`, and `OBSERVERS.md` for crate behavior.
- `CHANGE_RULES.md`, `TOOL_ROSTER.md`, and `TESTS.md` for maintenance policy.
