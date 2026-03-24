# bijux-dna-bench

## What this crate does
Benchmark comparisons over analyze/runtime artifacts producing decisions and summaries.

## What it must not do (boundaries)
No tool execution or planning.

## Role in the stack
Upstream: analyze outputs. Downstream: reports/CI.

## Public API / entrypoints
See `crates/bijux-dna-bench/docs/INDEX.md`, `crates/bijux-dna-bench/docs/BENCH_CONTRACT.md`, `crates/bijux-dna-bench/docs/BENCH_FORMAT.md`, `crates/bijux-dna-bench/docs/REPRODUCIBILITY.md`,
`crates/bijux-dna-bench/docs/LEGACY.md`, `crates/bijux-dna-bench/docs/CHANGE_RULES.md`.

## Benchmark contract
Inputs: analyze decisions + runtime observations.
Outputs: deterministic benchmark decisions and summaries.
See `crates/bijux-dna-bench/docs/BENCH_CONTRACT.md` and `crates/bijux-dna-bench/docs/BENCH_FORMAT.md`.

## Key contracts it owns/consumes
Owns the benchmark decision contract and consumes analyze/runtime artifact schemas.
See `crates/bijux-dna-bench/docs/BENCH_CONTRACT.md` and `crates/bijux-dna-bench/docs/BENCH_FORMAT.md`.

## Artifacts / Contracts
- `decision.json`
- `observations.jsonl`
- `summary.json`
Examples live in `crates/bijux-dna-bench/docs/BENCH_FORMAT.md`.

## Determinism
Only timestamps may vary; ordering and scores must not.
See `crates/bijux-dna-bench/docs/REPRODUCIBILITY.md` and `tests/determinism/*`.

## Effects & determinism guarantees
Pure comparison; deterministic outputs. See `crates/bijux-dna-bench/docs/EFFECTS.md` and the golden tests below.

## Why benchmark is not planner
Benchmark consumes analyze outputs and scores comparisons only; it never plans or stages work.
See the architecture boundary test in `tests/contracts/architecture.rs`.

## How to run its tests
See `crates/bijux-dna-bench/docs/TESTS.md`. Golden tests: `tests/contracts/bench_contract.rs`,
`tests/determinism/determinism.rs`, `tests/determinism/bench_realistic_snapshot.rs`,
`tests/contracts/contract_handshake.rs`.

## Start here in code
`src/summary.rs`, then `src/artifacts/writer.rs`.

## Where the docs live
Start at `crates/bijux-dna-bench/docs/INDEX.md` and follow the crate docs listed above.
Benchmark suite ownership and layout are documented in `bench/index.md`.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `crates/bijux-dna-bench/docs/CHANGE_RULES.md`.
