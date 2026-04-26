# bijux-dna-bench-model Architecture

`bijux-dna-bench-model` is the pure benchmark model crate. It owns typed
contracts, deterministic validation, comparison reports, gate policy decisions,
and statistical helpers. It does not run benchmarks, read or write artifacts, or
call product-facing APIs.

## Source Tree

```text
src/
├── lib.rs
├── compare/
│   ├── diff.rs
│   ├── report.rs
│   ├── stable_surface.rs
│   └── stratify.rs
├── contract/
│   ├── records.rs
│   ├── schema_versions.rs
│   └── suite/
│       ├── analysis.rs
│       ├── diversity.rs
│       ├── edge_ports.rs
│       ├── governance.rs
│       ├── graph.rs
│       ├── param_bindings.rs
│       └── validation/
├── diagnostics/
├── model/
│   ├── decision.rs
│   ├── graph.rs
│   ├── observation/
│   ├── suite/
│   │   └── support/
│   └── summary.rs
├── policy/
│   ├── gate_policy/
│   └── outcomes.rs
├── public_api/
└── stats/
    ├── bootstrap.rs
    ├── outlier_detection.rs
    └── robust_estimators/
```

## Module Roles

- `lib.rs` exposes only the curated public surface and named namespaces.
- `public_api/stable_surface.rs` is the stable export list used by consumers.
- `model/` owns serializable benchmark contracts: observations, suites,
  decisions, summaries, graph nodes, and suite support records.
- `contract/records.rs` validates observation, summary, and decision records.
- `contract/suite/` validates suite shape, stage governance, graph edges,
  parameter bindings, diversity requirements, and analysis requirements.
- `compare/` builds deterministic typed diffs between benchmark summaries.
- `policy/gate_policy/` evaluates configured thresholds, regression windows,
  required metrics, and per-stage overrides.
- `diagnostics/` owns the stable `BenchError` taxonomy.
- `stats/` owns deterministic robust estimators, seeded bootstrap intervals, and
  MAD-based outlier detection.

## Test Tree

```text
tests/
├── boundaries.rs
├── boundaries/
├── contracts.rs
├── contracts/
├── determinism.rs
├── determinism/
├── schemas.rs
├── schemas/
├── semantics.rs
├── semantics/
└── snapshots/
```

Boundary tests lock the crate root, docs allowance, source tree, and test tree.
Contract tests exercise validation behavior. Determinism tests protect seeded
and non-random behavior. Schema tests lock public exports and docs-linked API
expectations. Semantic tests cover explainability and metric meaning.

## Dependency Shape

Normal dependencies are limited to pure model support:

- `bijux-dna-core` for stable ids and shared contract primitives.
- `bijux-dna-stage-contract` for stage contract integration.
- `bijux-dna-domain-fastq` for stage/domain ids used by suite validation.
- `bijux-dna-analyze` for metric direction semantics.
- `serde`, `serde_json`, `anyhow`, and `fastrand` for serialization,
  validation errors, and seeded bootstrap sampling.

This crate must not depend on benchmark runners, CLI adapters, API crates,
runtime orchestration, persistence/report writers, or product execution crates.
