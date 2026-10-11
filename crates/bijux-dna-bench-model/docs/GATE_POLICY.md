# Gate Policy Contracts

This crate defines the typed `GatePolicy` input and `GateDecision` output
contracts. `bijux-dna-science` evaluates one benchmark summary row metric map
against a policy. Orchestration decides when evaluation runs and where the
result is persisted.

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

The science evaluator resolves metric direction through canonical metric
semantics unless the policy provides an override.

## Evaluator Contract

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

The policy and decision types are pure data contracts. They must not read
benchmark artifacts, invoke tools, run pipelines, write reports, or make API
decisions. Scientific acceptance logic must not be added to this crate.

## Verification

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-bench-model --test semantics --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-science --test benchmark_gate
```
