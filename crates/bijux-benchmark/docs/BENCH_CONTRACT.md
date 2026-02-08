# BENCH_CONTRACT

## Contract scope
Benchmark inputs are analyze outputs + runtime records.
Outputs are decisions and summaries compatible with analyze/runtime.

## Inputs
- `decision.json` from analyze (per-run decisions + rationale).
- `observations.jsonl` from runtime/analyze (metric stream).
- Optional run metadata for reproducibility (see `docs/REPRODUCIBILITY.md`).

## Output meaning
- decision: the selected tool or pipeline variant with justification.
- summary: aggregated comparison results used by CI or report layers.

## Reproducing a benchmark
1. Collect the input artifacts for a run pair.
2. Run the benchmark comparison (see `docs/REPRODUCIBILITY.md`).
3. Compare emitted `summary.json` and `decision.json` to fixtures.
