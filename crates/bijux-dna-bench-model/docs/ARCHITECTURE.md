# Architecture

`bijux-dna-bench-model` is the pure benchmark model crate. It owns typed
benchmark contracts, deterministic validation, comparison reports, gate policy
decisions, and statistical helpers.

## Source Map

- `src/public_api/` exposes the curated stable surface.
- `src/model/` owns observations, suites, decisions, summaries, graph nodes,
  and suite support records.
- `src/contract/` validates records, suite shape, stage governance, graph edges,
  parameter bindings, diversity requirements, and analysis requirements.
- `src/compare/` builds deterministic typed diffs between benchmark summaries.
- `src/policy/` evaluates gate thresholds, regression windows, required
  metrics, and per-stage overrides.
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
FASTQ domain identifiers, analysis metric semantics, serialization, validation
errors, and seeded sampling.

## Command Inventory

`docs/COMMANDS.md` lists model-level operations owned by this crate and must
stay aligned with public API and contract tests.
