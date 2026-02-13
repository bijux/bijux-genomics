# Explainability

## What
Explains how tool selection and defaults are reported.

## Why
Ensures scientific transparency.

## Non-goals
- Automatic decision‑making beyond recorded reasons.

## Contracts
- Explain output includes defaults diff, tool reasons, contract hashes.
- `explain.json` must include `decision_traces` as a required field.
- Each decision trace entry must include: `id`, `selected`, `evidence`, `source`.
- VCF explain output must include `decision.coverage_regime` and selected calling regime.
- VCF explain output must include regime observability fields:
  - `coverage_regime.selected`
  - `coverage_regime.thresholds_used`
  - `coverage_regime.observed_coverage_stats`

## Examples
- Planner records why fastp was selected for trimming.
- Planner records coverage evidence and selected VCF calling regime (`gl`, `pseudohaploid`, or `diploid`).
- Planner records applied threshold profile (for example `modern_wgs_shotgun`) and observed mean depth statistics.
- Use `./scripts/run.sh tooling simulate-coverage-regime <mean_depth_x> --profile <regime_profile>` to debug regime selection deterministically.

## Failure modes
- Missing reasons or hashes fails explainability checks.
- Missing `decision_traces` or missing `decision.coverage_regime` for VCF runs fails contract checks.
