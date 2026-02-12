# bijux-dna-bench

## What this crate does
Benchmark comparisons over analyze/runtime artifacts producing decisions and summaries.

## What it must not do (boundaries)
No tool execution or planning.

## Role in the stack
Upstream: analyze outputs. Downstream: reports/CI.

## Public API / entrypoints
See `docs/INDEX.md`, `docs/BENCH_CONTRACT.md`, `docs/BENCH_FORMAT.md`, `docs/REPRODUCIBILITY.md`,
`docs/LEGACY.md`, `docs/CHANGE_RULES.md`.

## Benchmark contract
Inputs: analyze decisions + runtime observations.
Outputs: deterministic benchmark decisions and summaries.
See `docs/BENCH_CONTRACT.md` and `docs/BENCH_FORMAT.md`.

## Key contracts it owns/consumes
Owns the benchmark decision contract and consumes analyze/runtime artifact schemas.
See `docs/BENCH_CONTRACT.md` and `docs/BENCH_FORMAT.md`.

## Artifacts / Contracts
- `decision.json`
- `observations.jsonl`
- `summary.json`
Examples live in `docs/BENCH_FORMAT.md`.

## Determinism
Only timestamps may vary; ordering and scores must not.
See `docs/REPRODUCIBILITY.md` and `tests/determinism/*`.

## Effects & determinism guarantees
Pure comparison; deterministic outputs. See `docs/EFFECTS.md` and the golden tests below.

## Why benchmark is not planner
Benchmark consumes analyze outputs and scores comparisons only; it never plans or stages work.
See the architecture boundary test in `tests/contracts/architecture.rs`.

## How to run its tests
See `docs/TESTS.md`. Golden tests: `tests/contracts/bench_contract.rs`,
`tests/determinism/determinism.rs`, `tests/determinism/bench_realistic_snapshot.rs`,
`tests/contracts/contract_handshake.rs`.

## Start here in code
`src/summary.rs`, then `src/artifacts/writer.rs`.

## Where the docs live
Start at `docs/INDEX.md` and follow the crate docs listed above.

## Failure modes
Primary failures surface as snapshot or contract violations; inspect the golden tests and referenced docs.

## Stability
Contract and behavior changes follow `docs/CHANGE_RULES.md`.
