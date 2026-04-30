# bijux-dna-bench

`bijux-dna-bench` turns governed benchmark observations into deterministic
summaries, gate decisions, comparisons, and persisted benchmark artifacts.

This crate follows repository governance documentation. `README.md` and
`README.md`; re-read those files before editing this child
repository and before committing.

## Scope

This crate owns:

- Loading benchmark suite specifications.
- Summarizing benchmark observations into stable `BenchmarkSummary` values.
- Evaluating summaries against `GatePolicy` rules.
- Comparing completed summaries.
- Reading runtime/analyze artifacts that are already materialized.
- Writing deterministic benchmark artifacts under declared output roots.
- Resolving crate-owned benchmark data and suite directories.

This crate does not plan workflows, execute tools, choose runner backends, or own
domain-stage semantics. Those boundaries belong to planner, runner, runtime,
domain, and API crates.

## Public Operations

The SSOT for callable operations is `docs/COMMANDS.md`.

- `load-suite`
- `load-corpus-manifest`
- `load-corpus-catalog`
- `load-bundle-manifest`
- `load-bundle-catalog`
- `summarize`
- `compare`
- `gate`
- `bench-data-dir`
- `bench-suites-dir`
- `bench-corpora-dir`
- `bench-bundles-dir`

## Architecture

- `src/lib.rs` stays thin and re-exports `public_api`.
- `src/public_api/` owns the curated stable surface.
- `src/workflow/` owns suite loading, summarization, comparison, gate decisions,
  and suite artifact persistence.
- `src/repo/` owns workspace path policy and read-only run artifact loading.
- `src/artifacts/` owns deterministic artifact serialization.
- `crates/bijux-dna-bench/bench/suites/` contains the checked-in benchmark suite catalog.
- `crates/bijux-dna-bench/bench/corpora/` contains checked-in benchmark corpus manifests.
- `crates/bijux-dna-bench/bench/bundles/` contains checked-in benchmark bundle manifests.

See `docs/ARCHITECTURE.md`, `docs/BOUNDARY.md`, and `docs/PUBLIC_API.md` for the
full contract.

## Documentation

The crate root intentionally has only this `README.md`. All other crate docs live
under `docs/`, with a 10-document allowance enforced by boundary tests:

- `docs/ARCHITECTURE.md`
- `docs/BENCH_CONTRACT.md`
- `docs/BENCH_FORMAT.md`
- `docs/BOUNDARY.md`
- `docs/CHANGE_RULES.md`
- `docs/COMMANDS.md`
- `docs/PUBLIC_API.md`
- `docs/REPRODUCIBILITY.md`
- `docs/SUITE_DESIGN.md`
- `docs/TESTS.md`

## Verification

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-bench --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench --test semantics --no-default-features
```
