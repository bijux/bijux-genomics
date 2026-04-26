# Public API

The crate root exports a narrow BAM stage surface from `src/lib.rs`. Internal
plugin modules remain private.

## Public Modules

- `metrics`
- `observer`
- `stage_specs`

## Stable Root Exports

- `BamStagePlugin`
- `StagePlanJson`
- `implemented_stages`

## Stage Specs Surface

`stage_specs` intentionally re-exports:

- `bijux_dna_domain_bam::*`
- `crate::metrics`
- `crate::observer`

This keeps planner-facing stage code on the BAM domain vocabulary without
pulling planner or runtime ownership into this crate.

## Internal Modules

- `plugin` materializes a planned BAM invocation and parses existing outputs.
- `surface` keeps crate-root aliases and registry functions out of `lib.rs`.

## Compatibility Rules

- Adding a parser, metric field, or stage contract requires matching snapshot
  and registry coverage.
- Removing or renaming a public export is breaking.
- Changing the shape of observer JSON, stage contract snapshots, or metrics
  envelopes is breaking unless versioned explicitly.
- New callable operations must be listed in `docs/COMMANDS.md`.

## Plugin Preconditions

`BamStagePlugin::materialize` validates that the stage ID belongs to the BAM
registry and that the planner-provided command template is present and nonblank.
The plugin does not choose a tool, construct a shell command, or resolve an
environment.

`BamStagePlugin::parse_outputs` parses existing artifact paths into a metrics
envelope and preserves reported artifact references. It must not write files or
execute tools.
