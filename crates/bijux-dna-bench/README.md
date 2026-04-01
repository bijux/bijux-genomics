# bijux-dna-bench

## What this crate does
Builds deterministic benchmark summaries, policy decisions, and comparison reports from benchmark
observations and analyze/runtime-derived artifacts.

## Boundaries
This crate does not plan workflows or execute tools. It owns benchmark orchestration and artifact
serialization only.

## Internal layout
- `src/public_api/`: curated stable surface
- `src/workflow/`: benchmark loading, summarization, evaluation, and suite orchestration
- `src/repo/`: repository root discovery plus persisted run-artifact loading
- `src/artifacts/`: deterministic benchmark artifact writers

## Public entrypoints
Start with `PUBLIC_API.md` and `docs/ARCHITECTURE.md`. The crate root is intentionally thin and
routes its stable surface through `src/public_api/mod.rs`.

## Contracts and operating rules
- benchmark contracts: `docs/BENCH_CONTRACT.md`
- artifact format: `docs/BENCH_FORMAT.md`
- reproducibility: `docs/REPRODUCIBILITY.md`
- architecture: `docs/ARCHITECTURE.md`
- test map: `docs/TESTS.md`

## Tests
See `docs/TESTS.md` for the full map. The test tree is organized by enduring intent:
- `tests/boundaries.rs`: source-tree guardrails
- `tests/contracts.rs`: API, contract, fixture, and workspace-path checks
- `tests/determinism.rs`: stable ordering and realistic snapshot checks
- `tests/semantics.rs`: gate semantics
