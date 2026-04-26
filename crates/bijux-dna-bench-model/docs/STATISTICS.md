# Statistics

`bijux-dna-bench-model` owns deterministic statistical helpers used by benchmark
summaries and gates.

## Robust Estimators

- `robust_stats` computes deterministic robust summary statistics.
- Median and MAD-based choices reduce sensitivity to heavy-tailed benchmark
  metrics.
- Sparse observations reduce confidence and must be reported by callers through
  low-power or policy evidence.

## Bootstrap Confidence Intervals

- `bootstrap_ci` is the only seeded sampling helper.
- The seed must be derived from stable ids or provided by a caller.
- Same values and same seed must produce identical confidence intervals.

## Outlier Detection

- `mad_outliers` detects median-absolute-deviation outliers.
- Outlier replicate ids must remain deterministic for the same input order and
  values.

## Assumptions

- Metrics are comparable within the same suite, stage, tool, and parameter
  grouping.
- Non-stationary metrics require normalization before they reach this crate.
- Unknown metrics can be carried in observations, but gate policies may reject
  unknown metric ids.
