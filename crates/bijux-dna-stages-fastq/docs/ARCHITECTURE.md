# Architecture

This file is a compact map for the FASTQ stages crate. Keep the contract here short and put detailed behavior in the focused docs listed below.

## Layout
- `lib.rs`, `contracts.rs`, and `surface.rs` expose the supported public surface.
- `stage_specs/` owns declarative stage and artifact descriptions.
- `runtime/` owns interpretation policy for stages and stage-tool pairs.
- `observer/` owns observer-facing parsing helpers and command support.
- `metrics/` owns governed envelope builders and the `stage_metrics/` namespace tree grouped by workflow family.
- `plugin/` owns plugin integration details, with `observation_context.rs` for observation state assembly and `output_contract.rs` for invariant, report, warning, and event assembly.
- `plugin/semantic/` owns semantic metric extraction grouped by workflow family instead of one catch-all parser.

## Change rules
- Keep stage specs declarative and free of command construction or execution.
- Keep runtime interpretation isolated from the public surface and catalog definitions.
- Keep public contract exports in `contracts.rs` instead of hiding them under unrelated query modules.
- Group metrics by concern instead of growing one catch-all module, and add new stage metrics under the closest `stage_metrics/` family module.
- Keep plugin parsing orchestration small by pushing context-building and output-contract assembly into focused helpers.

## Pointers
- `INDEX.md` for the documentation map.
- `STAGE_CONTRACTS.md`, `STAGE_LIST.md`, `OBSERVERS.md`, and `METRICS.md` for crate behavior.
- `CHANGE_RULES.md`, `TOOL_ROSTER.md`, and `TESTS.md` for maintenance policy.
