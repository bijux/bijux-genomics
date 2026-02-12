# REPRODUCIBILITY

## Reproduce a benchmark run
1. Collect `decision.json` + `observations.jsonl` + `summary.json` inputs.
2. Run the benchmark comparison via the crate entrypoint.
3. Compare outputs to fixtures in `tests/fixtures/*`.

## Determinism contract
Allowed to vary:
- timestamps embedded in metadata (if present).

Forbidden to vary:
- ordering of decisions, observations, and summaries.
- numerical scores and derived deltas.
- chosen tool or tie-break outcomes.

Determinism is enforced by `tests/determinism/*`.
