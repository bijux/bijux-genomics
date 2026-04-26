# bijux-dna-domain-vcf Domain Model

`bijux-dna-domain-vcf` owns VCF domain truth for typed contracts and downstream stage taxonomy.

## Stage Truth

- `stage_baseline.rs` owns the canonical call/filter/stats stage set.
- `taxonomy/` owns the downstream VCF stage taxonomy, support status, coverage regimes, forbidden
  transitions, and stage order.
- `coverage.rs` reports whether a stage or tool is contract-only, domain-only, or execution-ready.

## Parameters And Metrics

- `params/` owns typed VCF parameter models and effective parameter enums.
- `metrics.rs` owns schema-versioned VCF call summary, filter breakdown, and stats metrics.
- Public catalogs in `lib.rs` expose stable param and metric identifiers.

## Contracts

- `contracts/stage_io.rs` owns required inputs, outputs, and indexes.
- `contracts/stage_metrics.rs` owns per-stage metrics schemas and required fields.
- `contracts/stage_delivery.rs` owns output format guarantees.
- `contracts/panel_governance.rs` owns reference panel selection and validation.
- `contracts/invariants.rs` owns VCF and species-keyed invariant checks.

## Registry Materialization

`registry_emit.rs` returns deterministic TOML for generated config artifacts. It must not choose an
output path or write files.
