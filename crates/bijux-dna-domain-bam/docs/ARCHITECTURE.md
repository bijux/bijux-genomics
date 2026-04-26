# bijux-dna-domain-bam Architecture

`bijux-dna-domain-bam` is a pure library crate for BAM domain truth. It defines typed stage ids, effective params, metrics, stage contracts, artifact policies, and invariants consumed by planners and stage crates.

## Source layout

```text
src/
├── alignment.rs
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
├── stage_specs/
└── types/
```

## Responsibilities

- `stage_specs/` owns the ordered BAM stage model and stage contract JSON.
- `params/` owns typed effective params and default parameter shapes.
- `metrics/` owns typed metric models and deterministic parsers for small fixture formats.
- `invariants/` owns BAM domain rules and verdict mapping.
- `types/` owns shared BAM-specific identifiers, paths, metadata, and catalog constants.
- `defaults.rs` owns deterministic default and aDNA preset parameter JSON.

## Boundaries

- This crate provides types and deterministic parsing helpers to downstream crates.
- This crate does not choose tools, spawn processes, inspect environments, write generated configs, or execute pipeline stages.
- Runtime execution belongs in runner, runtime, stage, planner, or developer-control-plane crates.
