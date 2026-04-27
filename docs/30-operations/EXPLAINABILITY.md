# Explainability

## What
Explains how tool selection and defaults are reported from
[SCIENTIFIC_DECISIONS.md](../20-science/SCIENTIFIC_DECISIONS.md) and
[SCIENTIFIC_DEFAULTS.md](../20-science/SCIENTIFIC_DEFAULTS.md).

## Why
Ensures scientific transparency.

## Non-goals
- Automatic decision‑making beyond recorded reasons.

## Contracts
- Explain output includes defaults diff, tool reasons, contract hashes.
- Explain output remains paired with the machine-readable report contract in
  [REPORT_CONTRACT.md](REPORT_CONTRACT.md).
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
- Use `cargo run -q -p bijux-dna-dev -- tooling run simulate-coverage-regime <mean_depth_x> --profile <regime_profile>` to debug regime selection deterministically; the command inventory is published in
  [crates/bijux-dna-dev/docs/COMMANDS.md](../../crates/bijux-dna-dev/docs/COMMANDS.md).

## Failure modes
- Missing reasons or hashes fails explainability checks.
- Missing `decision_traces` or missing `decision.coverage_regime` for VCF runs fails contract checks.
