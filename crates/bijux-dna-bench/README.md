# bijux-dna-bench

## What this crate does
Builds deterministic benchmark summaries, policy decisions, and comparison reports from benchmark
observations and analyze/runtime-derived artifacts.

## What it must not do (boundaries)
This crate must not plan workflows or execute tools. It owns benchmark orchestration and artifact
serialization only.

## Effects & determinism guarantees
This crate deterministically derives benchmark artifacts from recorded observations and managed
analysis/runtime inputs.

## Internal layout
- `src/public_api/`: curated stable surface
- `src/workflow/`: benchmark loading, summarization, evaluation, and suite orchestration
- `src/repo/`: repository root discovery plus persisted run-artifact loading
- `src/artifacts/`: deterministic benchmark artifact writers

## Public API / entrypoints
Start with `PUBLIC_API.md` and `docs/ARCHITECTURE.md`. The crate root is intentionally thin and
routes its stable surface through `src/public_api/mod.rs`.

## Key contracts it owns/consumes
- benchmark contracts: `docs/BENCH_CONTRACT.md`
- artifact format: `docs/BENCH_FORMAT.md`
- reproducibility: `docs/REPRODUCIBILITY.md`
- architecture: `docs/ARCHITECTURE.md`
- test map: `docs/TESTS.md`

## Artifacts / Contracts
Benchmark report artifacts and wire contracts are defined in `docs/BENCH_CONTRACT.md` and
`docs/BENCH_FORMAT.md`.

## How to run its tests
See `docs/TESTS.md` for the full map. The test tree is organized by enduring intent:
- `tests/boundaries.rs`: source-tree guardrails
- `tests/contracts.rs`: API, contract, fixture, and workspace-path checks
- `tests/determinism.rs`: stable ordering and realistic snapshot checks
- `tests/semantics.rs`: gate semantics

## Failure modes
- invalid benchmark fixture/report schema inputs
- missing benchmark artifact dependencies for synthesis
- policy gate violations in benchmark package composition

## Where the docs live
Primary docs live in `docs/INDEX.md`; test coverage maps live in `docs/TESTS.md`.
