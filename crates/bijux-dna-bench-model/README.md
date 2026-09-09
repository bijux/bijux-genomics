# bijux-dna-bench-model

`bijux-dna-bench-model` owns pure benchmark model contracts, validation,
comparison, gate policy and decision records, diagnostics, and deterministic
statistical helpers.

This crate follows repository governance documentation. Re-read
[../../README.md](../../README.md) before editing this crate or making commits in this
repository.

## Scope

This crate owns:

- Benchmark observation, suite, summary, decision, graph, and policy types.
- Checksum-pinned corpus acquisition plans and admitted scientific data catalogs.
- Permanent benchmark run identities and append-only attempt lifecycle types.
- Schema ids, pure renderers, and the canonical checked-in persisted-schema registry.
- Canonical normalized-result records and execution-metrics envelope schemas.
- Deterministic comparison reports for benchmark summaries.
- Gate policy configuration, overrides, violations, and decision outcomes.
- Deterministic robust statistics, seeded bootstrap intervals, and outlier
  detection.
- Stable public exports through `src/public_api/stable_surface.rs`.

This crate does not own filesystem I/O, benchmark artifact writing, workflow
execution, runner integration, API orchestration, or report persistence.

## Managed Operations

`docs/COMMANDS.md` is the SSOT for callable pure-model operations:

- `validate-all-domain-expected-result-rows`
- `validate-all-domain-rendered-command-rows`
- `validate-all-domain-stage-tool-table-rows`
- `admit-real-smoke-core-subset`
- `validate-suite-with-catalog`
- `validate-suite`
- `validate-observation`
- `validate-summary`
- `validate-decision`
- `compare-summaries`
- `robust-stats`
- `bootstrap-ci`
- `mad-outliers`

## Architecture

- `src/lib.rs` exposes the curated public surface and focused namespaces.
- `src/public_api/` owns stable re-exports.
- `src/model/` owns benchmark contract data types, including normalized-result
  records and execution-metrics envelopes.
- `src/contract/` owns schema ids and validation rules.
- `src/policy/` owns gate configuration and decision records. Scientific gate
  evaluation lives in `bijux-dna-science`.
- `src/compare/` owns deterministic summary comparison.
- `src/stats/` owns deterministic statistical helpers.
- `src/diagnostics/` owns stable error taxonomy.

## Documentation

The crate root intentionally has only this `README.md`. All other crate docs live
under `docs/`, with a 10-document allowance enforced by boundary tests:

- `docs/ARCHITECTURE.md`
- `docs/BOUNDARY.md`
- `docs/CHANGE_RULES.md`
- `docs/COMMANDS.md`
- `docs/DECISION_EXPLAINABILITY.md`
- `docs/DETERMINISM.md`
- `docs/GATE_POLICY.md`
- `docs/PUBLIC_API.md`
- `docs/STATISTICS.md`
- `docs/TESTS.md`

## Verification

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-bench-model --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test semantics --no-default-features
```

Set `BIJUX_UPDATE_GENERATED_SCHEMAS=1` only when intentionally regenerating
checked-in schemas from the Rust model registry; the ordinary schema test is a
read-only drift gate.

## Catalog linkage

The default `canonical-fastq-catalog` feature preserves `contract::validate_suite` and the canonical FASTQ adapter. Pure record consumers can disable default features and use `contract::validate_suite_with_catalog` with an explicit provider. This removes the model-to-FASTQ compilation dependency for those builds. Full workspace feature unification may still enable the canonical adapter; a minimal consumer build is a separate measured configuration.

BAM scoring records, per-binding evidence selection, weighted ranking and report counts are owned by the model scoring policy. Callers supply typed observations; repository acquisition and TSV presentation remain adapter responsibilities.

BAM scoring admits correctness fractions in `[0, 1]` and finite nonnegative runtime/memory before ranking. Scientific metric columns require finite values while retaining their declared units and signed values; their format-specific range and provenance admission belongs to the artifact contract.
