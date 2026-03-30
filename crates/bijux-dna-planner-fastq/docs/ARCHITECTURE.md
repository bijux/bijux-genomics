# Architecture

`bijux-dna-planner-fastq` turns FASTQ planning inputs, policy, and stage-tool choices into
governed stage plans and execution graphs. The crate should read as a thin public surface over
focused subsystems instead of a pile of root-level files.

## Intended tree

```text
src/
  lib.rs
  surface.rs
  preprocess/
    mod.rs
    planning.rs
    policy.rs
  selection/
    mod.rs
    args.rs
    facade.rs
    tool_selection.rs
  planner/
    mod.rs
    benchmark.rs
    graph_policy.rs
    route_expansion.rs
    selection_planning.rs
    support.rs
    types.rs
  compose/
    mod.rs
    input_resolution.rs
    models.rs
    stage_params.rs
  tool_adapters/
    ...
    stages/transform/trim_reads/
      mod.rs
      config.rs
      reporting.rs
```

## Responsibilities

- `lib.rs` stays thin and re-exports the supported crate surface.
- `surface.rs` owns the public planner facade and crate-level stage constants.
- `preprocess/` owns pipeline choice and preprocess policy decisions.
- `selection/` owns tool allowlisting, override merging, and CLI-facing selection helpers.
- `planner/` owns graph planning, benchmarking, route expansion, and planner-local support.
- `compose/` owns stage-plan composition, explicit input resolution, and binding support models.
- `tool_adapters/` owns backend-specific stage plan construction.

## Design rules

- Root-level modules should be facades or stable subsystem entrypoints, not catch-all helpers.
- Selection logic should never rely on `include!`-based wiring.
- Composition helpers should be grouped by concern rather than accumulating in one file.
- Large stage adapter modules should split config/reporting support into named submodules.
