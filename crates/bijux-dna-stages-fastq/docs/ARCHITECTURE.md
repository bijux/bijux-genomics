# Architecture

This file is a compact map for the FASTQ stages crate. Keep the contract here short and put detailed behavior in the focused docs listed below.

## Layout
- `lib.rs` and `surface.rs` expose the supported public surface.
- `stage_specs/` owns declarative stage and artifact descriptions.
- `runtime/` owns interpretation policy for stages and stage-tool pairs.
- `observer/` owns observer-facing parsing helpers and command support.
- `metrics/` owns governed envelope builders grouped by concern.
- `plugin/` owns semantic interpretation and plugin integration details.

## Change rules
- Keep stage specs declarative and free of command construction or execution.
- Keep runtime interpretation isolated from the public surface and catalog definitions.
- Group metrics by concern instead of growing one catch-all module.

## Pointers
- `INDEX.md` for the documentation map.
- `STAGE_CONTRACTS.md`, `STAGE_LIST.md`, `OBSERVERS.md`, and `METRICS.md` for crate behavior.
- `CHANGE_RULES.md`, `TOOL_ROSTER.md`, and `TESTS.md` for maintenance policy.
