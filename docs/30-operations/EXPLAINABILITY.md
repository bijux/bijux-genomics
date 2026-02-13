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

## Examples
- Planner records why fastp was selected for trimming.
- Planner records coverage evidence and selected VCF calling regime (`gl`, `pseudohaploid`, or `diploid`).

## Failure modes
- Missing reasons or hashes fails explainability checks.
- Missing `decision_traces` or missing `decision.coverage_regime` for VCF runs fails contract checks.
