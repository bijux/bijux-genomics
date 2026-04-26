# Gate Policy

Gate policy evaluates one benchmark summary row metric map and returns a
deterministic `GateDecision`. The crate does not decide when a gate runs or
where the result is persisted.

## Policy Inputs

`GatePolicy` contains:

- `objective`: the optimization objective label.
- `required_metrics`: metrics that must be present for a complete decision.
- `thresholds`: metric thresholds evaluated by metric direction semantics.
- `allowed_regressions`: metric-specific regression windows.
- `must_not_regress`: metrics that must appear when regression protection is
  required.
- `semantics_overrides`: explicit metric direction overrides.
- `stage_overrides`: per-stage replacements for required metrics, thresholds,
  regression windows, and semantics overrides.

Metric direction comes from `bijux-dna-analyze` semantics unless the policy
provides an override.

## Decision Rules

- Required metrics that are absent are recorded in `missing_metrics`.
- Threshold checks compare observed values using the resolved metric direction:
  higher-is-better metrics must be at least the threshold, and lower-is-better
  metrics must be at most the threshold.
- Regression windows use the same metric direction semantics.
- A decision passes only when there are no violations and no missing metrics.
- `completeness_score` reflects required metric coverage.
- `rationale_trace` records deterministic evidence for missing metrics and
  threshold checks.

## Boundaries

Gate policy is pure in-memory evaluation. It must not read benchmark artifacts,
invoke tools, run pipelines, write reports, or make API decisions.

## Verification

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test semantics --no-default-features
```
