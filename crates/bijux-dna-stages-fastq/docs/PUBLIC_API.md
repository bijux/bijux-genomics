# Public API

The crate root exports a narrow FASTQ stage surface from `src/lib.rs`. Internal
plugin and runtime modules remain private.

## Public Modules

- `contracts`
- `metrics`
- `observer`
- `stage_specs`

## Stable Root Exports

- `FastqStagePlugin`
- `StagePlanJson`
- `RuntimeInterpretationLevel`
- `contract_stage_ids`
- `closed_execution_stage_ids`
- `implemented_stages`
- `observer_specialized_stage_ids`
- `observer_stage_ids`
- `observer_stage_tool_bindings`
- `runtime_interpretation_for_stage`
- `runtime_interpretation_for_stage_tool`
- `runtime_interpretation_stage_ids`

## Compatibility Rules

- Removing or renaming a public export is breaking.
- Changing observer JSON, stage contract snapshots, metrics envelopes, or
  runtime-interpretation classification is breaking unless versioned explicitly.
- Adding a parser, metric field, or stage contract requires matching fixture,
  snapshot, registry, and docs coverage.
- New callable operations must be listed in `docs/COMMANDS.md`.

## Internal Modules

- `plugin` validates FASTQ stage support, materializes planned invocations, and
  builds plugin output envelopes from existing artifacts.
- `runtime` classifies stage and stage-tool interpretation levels.
- `surface` keeps crate-root registry functions out of `lib.rs`.
