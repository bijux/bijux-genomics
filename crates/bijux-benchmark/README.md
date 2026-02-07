# bijux-benchmark

## What this crate does
Benchmark comparisons over analyze/runtime artifacts producing decisions and summaries.

## What it must not do (boundaries)
No tool execution or planning.

## Role in the stack
Upstream: analyze outputs. Downstream: reports/CI.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/BENCH_CONTRACT.md`, `docs/BENCH_FORMAT.md`, `docs/REPRODUCIBILITY.md`, `docs/LEGACY.md`, `docs/CHANGE_RULES.md`.

## Key contracts it owns/consumes
decision.json, observations.jsonl, summary.json.

## Effects & determinism guarantees
Pure comparison; deterministic outputs. See `docs/EFFECTS.md` and the golden tests below.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/bench_contract.rs`, `tests/determinism.rs`, `tests/bench_realistic_snapshot.rs`, `tests/contract_handshake.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
