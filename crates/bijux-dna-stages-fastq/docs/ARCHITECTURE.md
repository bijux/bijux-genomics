# Architecture

`bijux-dna-stages-fastq` is a stage-contract and observer library. It exposes
FASTQ contract registries, parser helpers, runtime-interpretation queries, and
stage-plugin output envelopes without owning planning, command construction,
process execution, or environment setup.

## Source Layout

```text
src/
├── contracts.rs
├── lib.rs
├── metrics/
│   ├── envelope_support.rs
│   ├── fastqc.rs
│   ├── filters.rs
│   ├── mod.rs
│   └── stage_metrics/
│       ├── analysis.rs
│       ├── analysis_feature_tables.rs
│       ├── analysis_screening.rs
│       ├── mod.rs
│       ├── reporting.rs
│       ├── transform.rs
│       ├── transform_filtering.rs
│       └── transform_pairing.rs
├── observer/
│   ├── artifacts.rs
│   ├── commands.rs
│   └── mod.rs
├── plugin/
│   ├── observation_context.rs
│   ├── output_contract.rs
│   ├── plugin_contracts.rs
│   └── semantic/
│       ├── feature_tables.rs
│       ├── mod.rs
│       ├── processing.rs
│       ├── processing_cleanup.rs
│       ├── processing_read_preparation.rs
│       ├── processing_trimming.rs
│       ├── profiling.rs
│       ├── quality.rs
│       ├── quality_qc.rs
│       ├── quality_read_flow.rs
│       ├── taxonomy.rs
│       └── validation_semantics.rs
├── runtime/
│   ├── interpretation.rs
│   └── mod.rs
├── stage_specs/
│   ├── artifacts.rs
│   ├── catalog.rs
│   └── mod.rs
└── surface.rs
```

## Ownership

- `contracts.rs` re-exports domain contract lookups and contract types.
- `surface.rs` owns stable registry and observer-surface queries.
- `stage_specs/` owns declarative stage and artifact descriptions.
- `runtime/` owns interpretation policy for stages and stage-tool pairs.
- `observer/` owns observer-facing parser helpers and crate-owned observer artifact writers.
- `metrics/` owns governed metrics envelope builders and stage-metrics families.
- `plugin/` validates FASTQ stage support, returns planned invocations, parses existing outputs,
  and assembles plugin output contracts.

## Change rules

- Keep stage specs declarative and free of command construction or execution.
- Keep runtime interpretation isolated from catalog definitions.
- Keep public contract exports in `contracts.rs` instead of hiding them under unrelated query modules.
- Group metrics by concern instead of growing one catch-all module, and add new stage metrics under the closest `stage_metrics/` family module.
- Keep plugin parsing orchestration small by pushing context-building and output-contract assembly into focused helpers.
- Update this file and architecture boundary tests together when the layout changes intentionally.
