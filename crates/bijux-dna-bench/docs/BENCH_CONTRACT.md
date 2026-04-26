# Benchmark Contract

`bijux-dna-bench` consumes benchmark suite specs and benchmark observations, then
emits summaries and gate decisions. Contract structs live in
`bijux-dna-bench-model`; this crate owns loading, summarization, comparison,
gating, and deterministic artifact persistence.

## Inputs

- `BenchmarkSuiteSpec` from checked-in TOML files under `crates/bijux-dna-bench/bench/suites/`.
- `BenchmarkObservation` records from runtime/analyze benchmark runs.
- Optional existing `observations.jsonl` when `BenchRunOptions.resume` is true.
- Runtime manifests and metric payloads that have already been produced by other
  crates.

## Outputs

- `BenchmarkSummary` for aggregated benchmark rows.
- `GateDecision` values for policy evaluation.
- `CompareReport` for summary-to-summary comparisons.
- Persisted artifacts documented in `docs/BENCH_FORMAT.md`.

## Required Invariants

- Suite specs must validate against `bijux.bench.suite.v1`.
- Observations must validate before summarization.
- Summary and decision artifacts must validate before being written.
- Output ordering must be deterministic.
- Gate decisions must include rationale and metric evidence.
- Benchmark operations must not execute tools or product workflows.

## Failure Modes

- Invalid suite schema or stage/tool ids.
- Missing or malformed observations.
- Missing required stratification metadata.
- Unknown metrics in gate policies.
- Contract validation failures before artifact persistence.
