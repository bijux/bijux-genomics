# bijux-dna-bench-model

`bijux-dna-bench-model` owns pure benchmark model contracts, validation,
comparison, gate policy evaluation, diagnostics, and deterministic statistical
helpers.

This crate follows repository governance documentation. `README.md` and
`README.md`; re-read those files before editing this child
repository and before committing.

## Scope

This crate owns:

- Benchmark observation, suite, summary, decision, graph, and policy types.
- Schema ids and validation entrypoints for benchmark model contracts.
- Deterministic comparison reports for benchmark summaries.
- Gate policy configuration, overrides, violations, and decision outcomes.
- Deterministic robust statistics, seeded bootstrap intervals, and outlier
  detection.
- Stable public exports through `src/public_api/stable_surface.rs`.

This crate does not own filesystem I/O, benchmark artifact writing, workflow
execution, runner integration, API orchestration, or report persistence.

## Managed Operations

`docs/COMMANDS.md` is the SSOT for callable pure-model operations:

- `validate-suite`
- `validate-observation`
- `validate-summary`
- `validate-decision`
- `compare-summaries`
- `gate-policy-decide`
- `robust-stats`
- `bootstrap-ci`
- `mad-outliers`

## Architecture

- `src/lib.rs` exposes the curated public surface and focused namespaces.
- `src/public_api/` owns stable re-exports.
- `src/model/` owns benchmark contract data types.
- `src/contract/` owns schema ids and validation rules.
- `src/policy/` owns gate configuration and evaluation.
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
