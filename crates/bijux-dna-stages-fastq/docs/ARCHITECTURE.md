# Architecture

`bijux-dna-stages-fastq` is a declarative FASTQ stage crate. It defines the closed
execution surface, the observer-facing parsing helpers, and the metrics builders
that convert observer outputs into governed envelopes.

## Intended tree

```text
src/
  lib.rs
  surface.rs
  runtime/
    mod.rs
    interpretation.rs
  stage_specs/
    mod.rs
    catalog.rs
    artifacts.rs
  observer/
    mod.rs
    artifacts.rs
    commands.rs
  metrics/
    mod.rs
    envelope_support.rs
    fastqc.rs
    filters.rs
    stage_metrics.rs
    stage_metrics_transform.rs
    stage_metrics_reporting.rs
    stage_metrics_analysis.rs
  plugin/
    mod.rs
    semantic/
      ...
```

## Module responsibilities

- `lib.rs` keeps the crate root thin and re-exports the supported public surface.
- `surface.rs` defines the crate-level execution and contract facade.
- `runtime/` contains runtime interpretation policy for stages and stage-tool pairs.
- `stage_specs/` owns declarative stage catalog and artifact descriptions.
- `observer/` owns observer-only parsing helpers plus command-facing helper wiring.
- `metrics/` owns envelope construction and metrics builders grouped by concern.
- `plugin/` owns semantic interpretation and plugin integration details.

## Design rules

- Stage specs stay declarative and must not build commands or execute tools.
- Observer helpers may parse tool outputs, but execution belongs outside this crate.
- Runtime interpretation stays isolated from the public surface and stage catalog.
- Metrics builders are grouped by concern instead of accumulating in one file.
