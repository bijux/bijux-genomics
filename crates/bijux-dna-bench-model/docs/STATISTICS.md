# Statistics

`bijux-dna-bench-model` owns deterministic statistical helpers for benchmark
summaries and gate inputs. These helpers do not decide workflow policy; they
produce typed values that callers can validate, compare, or gate.

## Robust Estimators

`stats::robust_stats(values)` returns `RobustStats`:

- `n`: number of observations.
- `median`: middle value after deterministic sorting.
- `mad`: median absolute deviation from the median.
- `iqr`: interquartile range from the sorted sample.
- `trimmed_mean`: mean after trimming 10 percent from each side.

Empty inputs return zero-valued stats with `n = 0`.

## Bootstrap Confidence Intervals

`stats::bootstrap_ci(values, samples, seed)` returns `BootstrapResult` with the
bootstrap mean, 2.5 percent lower interval, 97.5 percent upper interval, and
effective sample count.

Rules:

- The seed is required and must be stable for reproducible intervals.
- Empty `values` or zero `samples` return a zero-valued result with
  `samples = 0`.
- Confidence interval ordering must be stable for identical values, sample
  counts, and seed.

## Outlier Detection

`stats::mad_outliers(values, threshold)` returns deterministic outlier indices
using median absolute deviation. Indices refer to the original input order, so
callers can trace outliers back to replicate ids.

## Assumptions

- Values are already comparable within the same suite, dataset, stage, tool, and
  parameter grouping.
- Callers normalize non-stationary or unit-incompatible metrics before invoking
  this crate.
- Statistical helpers do not validate metric semantics; gate policy resolves
  metric direction separately.

## Verification

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --all-features
```
