# Architecture

This file stays short on purpose. `bijux-dna-planner-fastq` should read as a thin surface over named planning subsystems, with deeper rules documented next to the relevant contract docs.

## Layout
- `lib.rs` and `surface.rs` expose the supported planner surface.
- `preprocess/` owns pipeline choice and preprocess policy.
- `selection/` owns tool allowlisting, override merging, and selection helpers.
- `planner/` owns route expansion, graph planning, and planner-local support types.
- `compose/` owns stage-plan composition, input resolution, and parameter binding.
- `tool_adapters/` owns stage-specific plan construction.

## Change rules
- Keep root files as facades or stable subsystem entrypoints.
- Split adapter support by concern instead of growing catch-all modules.
- Update this map and the tree contract together when the layout changes intentionally.

## Pointers
- `INDEX.md` for the doc map.
- `PLANNER_MODEL.md`, `TOOL_SELECTION.md`, and `STAGE_MAPPING.md` for planner structure.
- `CHANGE_RULES.md`, `EFFECTS.md`, and `TESTS.md` for extension and verification policy.
