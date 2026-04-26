# Public API

The crate root exports the stable BAM stage surface from `src/lib.rs`.

## Public Modules
- `metrics`
- `observer`
- `stage_specs`

The `plugin` and `surface` modules are private implementation modules.

## Public Types And Functions
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

## Compatibility Rules
- Adding a parser, metric field, or stage contract requires matching snapshot
  and registry coverage.
- Removing or renaming a public export is breaking.
- Changing the shape of observer JSON, stage contract snapshots, or metrics
  envelopes is breaking unless versioned explicitly.
