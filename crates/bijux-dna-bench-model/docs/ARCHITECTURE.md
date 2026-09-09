# Architecture

`bijux-dna-bench-model` is the pure benchmark model crate. It owns typed
benchmark contracts, deterministic validation, comparison reports, gate policy
and decision records, and statistical helpers.

## Source Map

- `src/public_api/` exposes the curated stable surface.
- `src/model/` owns observations, suites, decisions, summaries, graph nodes, stage-profile data
  fitness, persisted container report-portfolio identities, permanent-run report dispositions,
  normalized-result records and execution-metrics envelopes, and suite support records.
- `src/model/local_pipeline_dag/` separates TOML/YAML configuration inputs from persisted validation reports; three configuration schemas share the model schema registry.
- `src/contract/` validates records, suite shape, stage governance, graph edges, data-fitness and
  portfolio closure, parameter bindings, container and benchmark report freshness, diversity
  requirements, and analysis requirements.
- `src/compare/` builds deterministic typed diffs between benchmark summaries.
- `src/policy/` owns typed gate configuration, violations, and decision records.
  `bijux-dna-science` owns scientific gate evaluation.
- `src/diagnostics/` owns the stable benchmark error taxonomy.
- `src/stats/` owns deterministic estimators, seeded bootstrap intervals, and
  MAD-based outlier detection.

## Test Map

- `tests/boundaries.rs` checks crate layout, docs allowance, dependency shape,
  and source/test taxonomy.
- `tests/contracts.rs` checks validation behavior.
- `tests/determinism.rs` protects seeded and non-random behavior.
- `tests/schemas.rs` locks public exports and docs-linked API expectations.
- `tests/semantics.rs` covers explainability and metric meaning.

## Boundaries

This crate does not run benchmarks, read or write benchmark artifacts, call the
CLI/API, or perform runtime orchestration. Execution and persistence live in
downstream crates.

## Dependency Direction

Normal dependencies are pure model support: core ids/contracts, stage contracts,
FASTQ and VCF domain identifiers and metrics contracts, analysis metric semantics,
serialization, validation errors, and seeded sampling.

## Command Inventory

`docs/COMMANDS.md` lists model-level operations owned by this crate and must
stay aligned with public API and contract tests.
