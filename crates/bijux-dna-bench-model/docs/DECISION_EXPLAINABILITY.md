# Decision Explainability

Benchmark decisions must be inspectable without re-running a benchmark. This
crate owns two explainability surfaces: versioned benchmark decisions and gate
policy decisions.

## BenchmarkDecision

`BenchmarkDecision` carries:

- `schema_version` fixed to `bijux.bench.decision.v1`.
- `stage_id`, `tool_id`, and `objective` identifying what was evaluated.
- `passes` as the final boolean outcome.
- `rationale`, an ordered list of `DecisionRationale` records.
- `missing_metrics`, an ordered list of required metrics absent from the input.

Each `DecisionRationale` records `metric_id`, `observed`, `direction`, `note`,
`weight`, and `contribution`. Callers must be able to explain why a metric
helped or hurt a decision from those fields alone.

## GateDecision

`GateDecision` carries:

- `schema_version` fixed to `bijux.bench.gate.v1`.
- `dataset_id`, `stage_id`, `tool_id`, and `params_hash` identity.
- `passes`, `violations`, `missing_metrics`, and `completeness_score`.
- `rationale_trace`, an ordered stable trace of threshold and missing-metric
  checks.

`GateViolation` records the failed `metric_id`, observed value, threshold, and
direction. Missing metrics are separated from threshold violations so callers can
distinguish absent evidence from failed evidence.

## Deterministic Ordering

Explainability records must remain stable for identical inputs:

- Use ordered maps or explicit sorting when metric order affects output.
- Keep `missing_metrics`, `violations`, and `rationale_trace` deterministic.
- Include enough identity fields that a persisted decision can be traced back to
  the evaluated suite row.

## Verification

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test semantics --no-default-features
```
