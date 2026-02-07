# BENCH_FORMAT

Benchmark artifacts written by `bijux-benchmark`.

## decision.json
- Decision outcome with selected tool/profile and rationale.
- Must include tool id, score, and gate decision.

## observations.jsonl
- Streaming observations per run/step.
- One JSON object per line (JSONL) with timestamps and metrics.

## summary.json
- Aggregated summary across runs.
- Contains totals, averages, and invariant checks.
