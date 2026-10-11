# bijux-dna-domain-bam Architecture

`bijux-dna-domain-bam` is a pure library crate for BAM domain truth. It defines typed stage ids, effective params, metrics, stage contracts, artifact policies, and invariants consumed by planners and stage crates.

## Source layout

```text
src/
├── alignment.rs
├── artifacts.rs
├── artifacts/
│   └── alignment/       # shared miniature document, SAM/BAM codecs and header policy
├── defaults.rs
├── invariants/
├── lib.rs
├── metrics/
│   ├── core/
│   ├── downstream/
│   └── pre/
├── params/
│   ├── core/
│   ├── downstream/
│   └── pre/
├── pipeline_contract.rs
├── prelude.rs
├── reference.rs
├── stage_specs/
└── types/
    └── artifact_reports/   # persisted schemas grouped by scientific responsibility
```

## Artifact schema boundary

`types/artifact_reports/` owns persisted identity, alignment, filtering, duplicate, coverage, recalibration, ancient DNA, population, workflow, resource and corpus models. Models depend on explicit sibling types and pure parameters/metrics; they do not import artifact processing or filesystem operations. Public names, schema identities and serialized fields remain stable through crate-root exports.

`artifacts/alignment/` owns the miniature document and format-specific readers/renderers, with shared header identity policy. SAM output publication delegates to infrastructure atomic writing; the codec has no dependency on scientific report evaluation; callers adapt reference names/lengths instead of passing their simulation model.

`artifacts.rs` still contains small-fixture transformations and report evaluation. Those effects are separate from the model modules and require further execution-owner separation; moving models does not create independent Cargo compilation units.

## Responsibilities

- `stage_specs/` owns the ordered BAM stage model and stage contract JSON.
- `alignment.rs` owns the stable artifact boundary exchanged with cross-domain pipeline stages.
- `reference.rs` owns deterministic BAM-header reference-span inspection.
- `params/` owns typed effective params and default parameter shapes.
- `metrics/` owns typed metric models and deterministic parsers for small fixture formats.
- `invariants/` owns BAM domain rules and verdict mapping.
- `types/` owns shared BAM-specific identifiers, paths, metadata, and catalog constants.
- `defaults.rs` owns deterministic default and aDNA preset parameter JSON.

## Boundaries

- This crate provides types and deterministic parsing helpers to downstream crates.
- Model modules do not choose tools, spawn processes, inspect environments, or write generated configs. Existing small-fixture transformation helpers still perform file IO through `artifacts/alignment/` and other existing helpers in `artifacts.rs`.
- Runtime execution belongs in runner, runtime, stage, planner, or developer-control-plane crates.
