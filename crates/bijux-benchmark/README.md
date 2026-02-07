# bijux-benchmark

## What this crate does
Compares runs and produces benchmark decisions and summaries from analyze/runtime artifacts.

## What it must not do (boundaries)
Must not execute tools or plan graphs. It consumes artifacts only.

## Public API / entrypoints
Benchmark contracts documented in `docs/BENCH_CONTRACT.md` and `docs/BENCH_FORMAT.md`.

## Key contracts it owns/consumes
Consumes run records and analyze reports; produces decision and summary outputs.

## Effects & determinism guarantees
Deterministic comparisons are enforced by snapshot tests. See `docs/REPRODUCIBILITY.md`.

## How to run its tests
See `docs/TESTS.md`. Key tests: `tests/bench_contract.rs`, `tests/determinism.rs`, `tests/bench_realistic_snapshot.rs`.

## Where the docs live
Start at `docs/INDEX.md`, then read `docs/BENCH_FORMAT.md`, `docs/REPRODUCIBILITY.md`, and `docs/LEGACY.md`.

## Artifacts / Contracts
Produces `decision.json`, `observations.jsonl`, `summary.json` under benchmark bundles.

## Failure modes
Schema mismatches or nondeterminism fail benchmark snapshot tests.

## Stability
Benchmark contracts are snapshot-tested; see `docs/CHANGE_RULES.md`.
