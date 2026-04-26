# bijux-dna-bench-model Boundary Contract

Owner: benchmark model contracts and pure decision logic.

## Owns

- Benchmark model records: suites, observations, summaries, decisions, graph
  nodes, stages, datasets, parameter bindings, and support requirements.
- Schema ids and validation entrypoints for suite, observation, summary, and
  gate decision records.
- Deterministic summary comparison reports.
- Gate policy configuration, per-stage overrides, violation records, and
  decision outcomes.
- Robust statistics, seeded bootstrap confidence intervals, and MAD outlier
  detection.
- Stable public exports through `src/public_api/stable_surface.rs`.

## Does Not Own

- Benchmark execution, process management, or runner backends.
- CLI commands, API route handlers, product orchestration, or persistence.
- Filesystem writes, report rendering, artifact publication, or network access.
- Domain-specific stage implementation beyond validating supported ids and
  contract shapes.

## Allowed Inputs

- Typed benchmark payloads built from trusted callers or fixtures.
- Stable suite, stage, dataset, tool, metric, and parameter ids.
- Explicit seeds for bootstrap operations.
- Policy thresholds, regression windows, required metrics, and per-stage
  overrides.

## Forbidden Dependencies

This crate must not depend on downstream orchestration or presentation layers:

- `bijux-dna-api`
- `bijux-dna-bench`
- `bijux-dna-runner`
- runtime or product execution crates
- CLI adapter crates
- report writer or artifact persistence crates

Allowed workspace dependencies must remain model-facing: core ids, stage
contracts, domain ids used by validation, analyze metric semantics, policies in
tests, and testkit fixtures.

## Effects

All crate-owned operations are pure in-memory computation. The only randomness
is `stats::bootstrap_ci`, and callers must provide the seed. Tests, examples,
and local commands may read fixtures through the Rust test harness, but library
code must not perform hidden I/O.

## Validation

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-bench-model --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test boundaries --no-default-features
```
