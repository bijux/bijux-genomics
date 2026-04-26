# Architecture

`bijux-dna-stages-bam` is a stage-contract library. It exposes BAM stage IDs and
stage-plugin behavior without owning planning, runtime scheduling, tool
selection, or command execution.

## Source Layout

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
├── observer.rs
├── plugin/
│   ├── invocation.rs
│   ├── mod.rs
│   └── output/
│       ├── collected_metrics.rs
│       ├── envelope.rs
│       └── mod.rs
├── stage_specs.rs
└── surface.rs
```

## Ownership

- `surface.rs` owns stable crate-root aliases and registry functions.
- `stage_specs.rs` re-exports BAM domain vocabulary for planner-facing stage code.
- `observer.rs` re-exports BAM parser functions owned by the BAM domain crate.
- `metrics/` discovers known output files in a stage output directory and folds them into
  `BamMetricsV1`.
- `plugin/` checks BAM stage support, materializes the planned invocation, parses existing
  outputs, and builds a metrics envelope.

## Stage Phases

- Pre stages validate, align, and summarize initial BAM quality.
- Core stages filter, mark duplicates, measure complexity, coverage, insert size, GC bias,
  and endogenous content.
- Downstream stages cover damage, authenticity, contamination, sex inference, bias mitigation,
  recalibration, haplogroups, genotyping, and kinship when enabled by upstream planning.

## Change rules

- Add root files only for enduring crate-level concerns.
- Keep metric parsing grouped by BAM metric concern.
- Keep plugin responsibilities separated between invocation, metric collection, and envelope
  construction.
- Update this file and `tests/boundaries/architecture.rs` together when the layout changes
  intentionally.
