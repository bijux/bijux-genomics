# Reproducibility

Benchmark results must be reproducible from the suite spec, observations,
policy, and options used for a run.

## Reproduce A Benchmark Summary

1. Load the suite TOML from `bench/suites/`.
2. Collect the governed `BenchmarkObservation` records for the run.
3. Use the same `BenchRunOptions`, including output directory and resume setting.
4. Call `summarize`.
5. Compare the emitted `BenchmarkSummary` with `summary.json` or test fixtures.

## Reproduce Gate Decisions

1. Recreate the `GatePolicy`.
2. Call `gate(policy, summary)` with the reproduced summary.
3. Compare `decision.json` and `decisions.json` against fixtures or persisted
   artifacts.

## Reproduce Comparisons

1. Load two completed `BenchmarkSummary` values.
2. Call `compare(summary_a, summary_b)`.
3. Compare the deterministic report against the stored compare snapshot.

## Allowed Variation

- Output directory paths may differ.
- Existing observations may be merged only when `BenchRunOptions.resume` is true.
- Test-local temporary directories may differ.

## Forbidden Variation

- Suite validation result.
- Observation grouping.
- Summary row ordering.
- Gate decision ordering.
- Numeric score, delta, outlier, and bootstrap results for the same inputs.
- Chosen pass/fail decision and rationale for the same policy.

Determinism is enforced by `tests/determinism.rs` and tests under
`tests/determinism/`.
