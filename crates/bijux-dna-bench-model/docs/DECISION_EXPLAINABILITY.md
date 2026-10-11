# Decision Explainability

Benchmark decisions must be inspectable without re-running a benchmark. The
model crate owns one persisted explainability surface: gate policy decisions.

## GateDecision

`GateDecision` is the sole benchmark decision contract and carries:

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
